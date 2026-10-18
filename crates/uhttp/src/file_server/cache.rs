//! In-memory response cache for the file server.
//!
//! A [`FileServerCache`] stores fully rendered responses (after content
//! encoding) so subsequent requests can be served without touching the
//! filesystem or re-compressing. Entries can be revoked individually, by path
//! or path prefix, or wholesale via [`FileServerCache::clear`].

use std::collections::BTreeMap;
use std::collections::HashMap;
use std::io;
use std::path::Path;
use std::path::PathBuf;
use std::pin::Pin;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::MutexGuard;
use std::sync::atomic::AtomicU64;
use std::sync::atomic::Ordering;
use std::task::Context;
use std::task::Poll;

use http::StatusCode;
use tokio::io::AsyncWrite;

use super::encoding::Encoding;

/// Identifies a cached response.
///
/// The negotiated content encoding is part of the key so the compressed and
/// uncompressed variants of the same file do not collide.
pub(crate) type CacheKey = (PathBuf, Encoding);

/// A rendered response that can be replayed to a client verbatim.
#[derive(Clone, Debug)]
pub struct CachedResponse {
  /// Response status
  pub status: StatusCode,
  /// Value for the `Content-Type` header
  pub mime_type: String,
  /// Value for the `Content-Encoding` header, if the body is compressed
  pub content_encoding: Option<&'static str>,
  /// Value for the `ETag` header, if one was computed
  pub etag: Option<String>,
  /// The (possibly compressed) response body
  pub body: Arc<[u8]>,
}

impl CachedResponse {
  /// Number of bytes in the (possibly compressed) body.
  pub fn len(&self) -> usize {
    self.body.len()
  }

  /// Whether the body is empty.
  pub fn is_empty(&self) -> bool {
    self.body.is_empty()
  }
}

/// Tunables for a [`FileServerCache`].
#[derive(Debug, Clone, Copy)]
pub struct FileServerCacheOptions {
  /// Total cached body bytes held before the least recently used entries are
  /// evicted.
  pub max_bytes: usize,
  /// Responses whose body exceeds this are never cached, keeping large files
  /// on the streaming path.
  pub max_entry_bytes: usize,
}

impl Default for FileServerCacheOptions {
  fn default() -> Self {
    Self {
      max_bytes: 64 * 1024 * 1024,
      max_entry_bytes: 1024 * 1024,
    }
  }
}

struct Entry {
  response: CachedResponse,
  /// Generation this entry was written under. Entries from an older generation
  /// are ignored and dropped lazily.
  generation: u64,
  /// Monotonic recency stamp used for LRU eviction.
  access: u64,
}

struct Inner {
  entries: HashMap<CacheKey, Entry>,
  /// Recency stamp -> key. The lowest stamp is the least recently used.
  lru: BTreeMap<u64, CacheKey>,
  next_access: u64,
  total_bytes: usize,
}

/// Shared, revokable in-memory cache of rendered file server responses.
///
/// Construct one and pass it to [`crate::file_server::FileServerOptions`] via
/// the `cache` field. Keep an [`Arc`] clone to invalidate entries from
/// application code, e.g. after redeploying static assets.
///
/// ```rust
/// use std::path::Path;
/// use std::sync::Arc;
///
/// use uhttp::file_server::FileServerCache;
///
/// # fn main() {
/// let cache = Arc::new(FileServerCache::new());
///
/// // later, after a deploy
/// cache.clear();
///
/// // or just the assets that changed
/// cache.remove_prefix(Path::new("/static"));
/// # }
/// ```
pub struct FileServerCache {
  inner: Mutex<Inner>,
  generation: AtomicU64,
  max_bytes: usize,
  max_entry_bytes: usize,
}

impl Default for FileServerCache {
  fn default() -> Self {
    Self::with_options(FileServerCacheOptions::default())
  }
}

impl FileServerCache {
  /// Create a cache using [`FileServerCacheOptions::default`].
  pub fn new() -> Self {
    Self::default()
  }

  /// Create a cache with explicit limits.
  pub fn with_options(options: FileServerCacheOptions) -> Self {
    Self {
      inner: Mutex::new(Inner {
        entries: HashMap::new(),
        lru: BTreeMap::new(),
        next_access: 0,
        total_bytes: 0,
      }),
      generation: AtomicU64::new(0),
      max_bytes: options.max_bytes,
      max_entry_bytes: options.max_entry_bytes,
    }
  }

  /// The largest response body this cache will store.
  pub fn max_entry_bytes(&self) -> usize {
    self.max_entry_bytes
  }

  /// The current generation.
  ///
  /// Capture this before rendering a response and pass it to [`Self::set`] so a
  /// response generated just before a [`Self::clear`] cannot resurrect stale
  /// data.
  pub(crate) fn generation(&self) -> u64 {
    self.generation.load(Ordering::Acquire)
  }

  /// Look up a response, refreshing its LRU position on a hit.
  pub(crate) fn get(
    &self,
    key: &CacheKey,
  ) -> Option<CachedResponse> {
    let mut inner = self.lock();
    let generation = self.generation();

    let (entry_generation, old_access) = {
      let entry = inner.entries.get(key)?;
      (entry.generation, entry.access)
    };

    if entry_generation != generation {
      remove_entry(&mut inner, key);
      return None;
    }

    let access = inner.next_access;
    inner.next_access += 1;
    inner.lru.remove(&old_access);
    inner.lru.insert(access, key.clone());
    let entry = inner.entries.get_mut(key)?;
    entry.access = access;

    Some(entry.response.clone())
  }

  /// Store a response.
  ///
  /// Ignored if `generation` no longer matches the cache's current generation,
  /// or if the body is too large to cache.
  pub(crate) fn set(
    &self,
    key: CacheKey,
    response: CachedResponse,
    generation: u64,
  ) {
    if response.body.len() > self.max_entry_bytes {
      return;
    }

    let mut inner = self.lock();
    if generation != self.generation() {
      return;
    }

    remove_entry(&mut inner, &key);

    let size = response.body.len();
    let access = inner.next_access;
    inner.next_access += 1;
    inner.total_bytes += size;
    inner.lru.insert(access, key.clone());
    inner.entries.insert(
      key,
      Entry {
        response,
        generation,
        access,
      },
    );

    evict(&mut inner, self.max_bytes);
  }

  /// Remove every cached response for a resolved path (all encodings).
  ///
  /// Returns the number of entries removed.
  pub fn remove_path(
    &self,
    path: &Path,
  ) -> usize {
    self.remove_where(|candidate| candidate == path)
  }

  /// Remove every cached response whose resolved path starts with `prefix`.
  ///
  /// Returns the number of entries removed. Useful for revoking a subtree of
  /// static assets, e.g. `Path::new("/static")`.
  pub fn remove_prefix(
    &self,
    prefix: &Path,
  ) -> usize {
    self.remove_where(|candidate| candidate.starts_with(prefix))
  }

  /// Drop every entry, returning the number that were removed.
  ///
  /// Bumps the generation so in-flight renders started before this call will
  /// not repopulate the cache with stale responses.
  pub fn clear(&self) -> usize {
    let mut inner = self.lock();
    let removed = inner.entries.len();
    inner.entries.clear();
    inner.lru.clear();
    inner.total_bytes = 0;
    self.generation.fetch_add(1, Ordering::AcqRel);
    removed
  }

  /// Number of entries currently held.
  pub fn len(&self) -> usize {
    self.lock().entries.len()
  }

  /// Whether the cache is empty.
  pub fn is_empty(&self) -> bool {
    self.len() == 0
  }

  /// Total bytes of cached response bodies.
  pub fn total_bytes(&self) -> usize {
    self.lock().total_bytes
  }

  fn remove_where(
    &self,
    predicate: impl Fn(&Path) -> bool,
  ) -> usize {
    let mut inner = self.lock();
    let keys: Vec<CacheKey> = inner
      .entries
      .keys()
      .filter(|(path, _)| predicate(path))
      .cloned()
      .collect();
    for key in &keys {
      remove_entry(&mut inner, key);
    }
    keys.len()
  }

  fn lock(&self) -> MutexGuard<'_, Inner> {
    self
      .inner
      .lock()
      .unwrap_or_else(std::sync::PoisonError::into_inner)
  }
}

fn remove_entry(
  inner: &mut Inner,
  key: &CacheKey,
) -> bool {
  match inner.entries.remove(key) {
    Some(entry) => {
      inner.lru.remove(&entry.access);
      inner.total_bytes = inner.total_bytes.saturating_sub(entry.response.body.len());
      true
    }
    None => false,
  }
}

fn evict(
  inner: &mut Inner,
  max_bytes: usize,
) {
  while inner.total_bytes > max_bytes {
    let Some((&access, _)) = inner.lru.iter().next() else {
      break;
    };
    let Some(key) = inner.lru.remove(&access) else {
      break;
    };
    if let Some(entry) = inner.entries.remove(&key) {
      inner.total_bytes = inner.total_bytes.saturating_sub(entry.response.body.len());
    }
  }
}

/// An [`AsyncWrite`] adapter that forwards bytes to an inner writer while
/// simultaneously buffering them, up to an optional limit.
///
/// Used to capture a streamed response so it can be inserted into the cache
/// without buffering unbounded data: once the limit is exceeded, buffering
/// stops and the response simply is not cached.
pub(crate) struct TeeWriter<W> {
  inner: W,
  buffer: Vec<u8>,
  limit: Option<usize>,
  overflowed: bool,
}

impl<W> TeeWriter<W> {
  pub(crate) fn new(
    inner: W,
    limit: Option<usize>,
  ) -> Self {
    Self {
      inner,
      buffer: Vec::new(),
      limit,
      overflowed: false,
    }
  }

  /// Consume the writer, returning the buffered bytes if the whole stream was
  /// captured within the limit.
  pub(crate) fn into_buffered(self) -> Option<Vec<u8>> {
    match self.limit {
      Some(_) if !self.overflowed => Some(self.buffer),
      _ => None,
    }
  }
}

impl<W> AsyncWrite for TeeWriter<W>
where
  W: AsyncWrite + Unpin,
{
  fn poll_write(
    self: Pin<&mut Self>,
    cx: &mut Context<'_>,
    buf: &[u8],
  ) -> Poll<Result<usize, io::Error>> {
    let this = self.get_mut();
    let written = std::task::ready!(Pin::new(&mut this.inner).poll_write(cx, buf))?;

    if let Some(limit) = this.limit
      && !this.overflowed
    {
      if this.buffer.len() + written > limit {
        this.overflowed = true;
        this.buffer = Vec::new();
        this.buffer.shrink_to_fit();
      } else {
        this.buffer.extend_from_slice(&buf[..written]);
      }
    }

    Poll::Ready(Ok(written))
  }

  fn poll_flush(
    self: Pin<&mut Self>,
    cx: &mut Context<'_>,
  ) -> Poll<Result<(), io::Error>> {
    let this = self.get_mut();
    Pin::new(&mut this.inner).poll_flush(cx)
  }

  fn poll_shutdown(
    self: Pin<&mut Self>,
    cx: &mut Context<'_>,
  ) -> Poll<Result<(), io::Error>> {
    let this = self.get_mut();
    Pin::new(&mut this.inner).poll_shutdown(cx)
  }
}

#[cfg(test)]
mod tests {
  use std::path::Path;
  use std::path::PathBuf;
  use std::sync::Arc;

  use http::StatusCode;

  use super::super::encoding::Encoding;
  use super::CacheKey;
  use super::CachedResponse;
  use super::FileServerCache;
  use super::FileServerCacheOptions;

  fn key(
    path: &str,
    encoding: Encoding,
  ) -> CacheKey {
    (PathBuf::from(path), encoding)
  }

  fn response(body: &[u8]) -> CachedResponse {
    CachedResponse {
      status: StatusCode::OK,
      mime_type: "text/plain".to_string(),
      content_encoding: None,
      etag: Some("abc".to_string()),
      body: Arc::from(body.to_vec()),
    }
  }

  #[test]
  fn stores_and_retrieves_a_response() {
    let cache = FileServerCache::new();
    let generation = cache.generation();
    let key = key("/index.html", Encoding::Identity);

    cache.set(key.clone(), response(b"hello"), generation);

    let cached = cache.get(&key).expect("cache hit");
    assert_eq!(&*cached.body, b"hello");
    assert_eq!(cached.mime_type, "text/plain");
    assert_eq!(cache.total_bytes(), 5);
  }

  #[test]
  fn does_not_store_bodies_over_the_entry_limit() {
    let cache = FileServerCache::with_options(FileServerCacheOptions {
      max_bytes: 1024,
      max_entry_bytes: 4,
    });
    let generation = cache.generation();
    let key = key("/big.bin", Encoding::Identity);

    cache.set(key.clone(), response(&[0u8; 5]), generation);

    assert!(cache.is_empty());
    assert!(cache.get(&key).is_none());
  }

  #[test]
  fn evicts_least_recently_used_entries_when_over_budget() {
    let cache = FileServerCache::with_options(FileServerCacheOptions {
      max_bytes: 10,
      max_entry_bytes: 100,
    });
    let generation = cache.generation();
    let first = key("/a", Encoding::Identity);
    let second = key("/b", Encoding::Identity);

    cache.set(first.clone(), response(&[0u8; 6]), generation);
    cache.set(second.clone(), response(&[0u8; 6]), generation);

    assert!(
      cache.get(&first).is_none(),
      "oldest entry should be evicted"
    );
    assert!(cache.get(&second).is_some());
    assert_eq!(cache.total_bytes(), 6);
  }

  #[test]
  fn clear_revokes_entries_and_bumps_generation() {
    let cache = FileServerCache::new();
    let key = key("/index.html", Encoding::Identity);
    cache.set(key.clone(), response(b"hello"), cache.generation());

    assert_eq!(cache.len(), 1);
    assert_eq!(cache.clear(), 1);
    assert!(cache.get(&key).is_none());
  }

  #[test]
  fn ignores_writes_from_a_stale_generation() {
    let cache = FileServerCache::new();
    let stale = cache.generation();
    cache.clear();

    cache.set(
      key("/index.html", Encoding::Identity),
      response(b"hello"),
      stale,
    );

    assert!(cache.is_empty());
  }

  #[test]
  fn removes_entries_by_path_prefix() {
    let cache = FileServerCache::new();
    let generation = cache.generation();
    cache.set(
      key("/static/a.js", Encoding::Identity),
      response(b"a"),
      generation,
    );
    cache.set(
      key("/static/b.js", Encoding::Identity),
      response(b"b"),
      generation,
    );
    cache.set(
      key("/other/c.js", Encoding::Identity),
      response(b"c"),
      generation,
    );

    assert_eq!(cache.remove_prefix(Path::new("/static")), 2);
    assert_eq!(cache.len(), 1);
    assert!(
      cache
        .get(&key("/static/a.js", Encoding::Identity))
        .is_none()
    );
    assert!(cache.get(&key("/other/c.js", Encoding::Identity)).is_some());
  }
}
