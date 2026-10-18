use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

#[cfg(feature = "file_server_transpile")]
use base64::Engine as _;
use tokio::io::AsyncWriteExt;

use super::cache::CachedResponse;
use super::cache::FileServerCache;
use super::cache::TeeWriter;
use super::compression::brotli_stream;
use super::compression::gzip_stream;
use super::compression::zstd_stream;
use super::encoding::Encoding;
use super::encoding::EncodingResolver;
use super::etag::EtagResolver;
use super::fallback_file::FallbackFile;
use super::fallback_file::FallbackFileResolveOptions;
use super::file_system::AsyncFileSystem;
use super::resolver::FileServerResolver;
use super::resolver::FileServerResolverOptions;
#[cfg(feature = "file_server_transpile")]
use super::transpile::TransformerContext;
#[cfg(feature = "file_server_transpile")]
use super::transpile::tsconfig::TsConfigCache;
#[cfg(feature = "file_server_transpile")]
use super::transpile::typescript;
use crate::HandleFunc;
use crate::Request;
use crate::ResponseBody;
use crate::StatusCode;
use crate::file_server::compression::uncompressed_stream;
use crate::headers;

/// Size of the buffer bodies are streamed through on their way to the client
pub const BODY_BUFFER_SIZE: usize = 64 * 1024;

/// Content type responses carrying transpiled TypeScript are served with
#[cfg(feature = "file_server_transpile")]
const JAVASCRIPT_CONTENT_TYPE: &str = "application/javascript; charset=utf-8";

#[derive(Debug, Default)]
pub enum ETagStrategy {
  /// Non cryptographic hash of the file, slower
  Hash,
  /// Faster
  #[default]
  LastModified,
  /// No etag calc
  Disabled,
}

#[cfg(feature = "file_server_transpile")]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct TranspileOptions {}

/// TypeScript extensions that are JIT transpiled, mapped to the syntax the
/// source is parsed as.
#[cfg(feature = "file_server_transpile")]
fn transpile_kind(path: &std::path::Path) -> Option<&'static str> {
  match path.extension().and_then(|extension| extension.to_str()) {
    Some("ts" | "mts" | "cts") => Some("ts"),
    Some("tsx") => Some("tsx"),
    _ => None,
  }
}

pub struct FileServerOptions {
  /// The filesystem files are read from
  pub file_system: Arc<dyn AsyncFileSystem>,
  /// The root directory to get files from
  pub root_dir: PathBuf,
  /// Send back compressed responses
  pub compress: bool,
  /// How to supply etag
  pub etag: ETagStrategy,
  /// Add Custom Headers
  pub custom_headers: HashMap<String, String>,
  /// Optional in-memory cache of rendered responses.
  pub cache: Option<Arc<FileServerCache>>,
  /// Relative path to fallback URL. Defaults to "404.html"
  pub fallback_route: Option<String>,
  /// Defaults to [`crate::StatusCode::NOT_FOUND] (404)
  pub fallback_status: Option<StatusCode>,
  /// Try to resolve html files on paths without specified files
  ///  `/`    -> `/index.html`
  ///  `/foo` -> `/foo/index.html` | `/foo.html`
  pub implicit_extensions: bool,
  #[cfg(feature = "file_server_transpile")]
  /// JIT Transpilation of TypeScript and SCSS sources
  pub transpile: Option<TranspileOptions>,
}

/// Serve files from the filesystem
pub fn create(options: FileServerOptions) -> crate::Result<HandleFunc> {
  let options = Arc::new(options);

  let resolver = Arc::new(FileServerResolver::new(FileServerResolverOptions {
    file_system: &options.file_system,
    root_dir: &options.root_dir,
    implicit_extensions: options.implicit_extensions,
  }));

  let etag_resolver = Arc::new(EtagResolver::new(&options.etag));

  let encoding_resolver = Arc::new(EncodingResolver::new(options.compress));

  let fallback = Arc::new(FallbackFile::resolve(FallbackFileResolveOptions {
    resolver: &resolver,
    fallback_route: &options.fallback_route,
    fallback_status: &options.fallback_status,
  }));

  #[cfg(feature = "file_server_transpile")]
  let tsconfig_cache = Arc::new(TsConfigCache::new());

  Ok(Box::new(move |req, mut res| {
    let options = Arc::clone(&options);
    let resolver = Arc::clone(&resolver);
    let etag_resolver = Arc::clone(&etag_resolver);
    let encoding_resolver = Arc::clone(&encoding_resolver);
    let fallback = Arc::clone(&fallback);
    #[cfg(feature = "file_server_transpile")]
    let tsconfig_cache = Arc::clone(&tsconfig_cache);

    Box::pin(async move {
      let resolved = resolver.resolve(req.uri.path()).await?;
      let file_path = resolved.path;

      // JIT transpile TypeScript sources. The request names the source file
      // directly; the transpiled module is served from memory and requests for
      // a sibling `.js` file are never resolved to it here.
      #[cfg(feature = "file_server_transpile")]
      if let Some(_transpile_options) = options.transpile
        && let Some(kind) = transpile_kind(&file_path)
        && let Ok(content) = options.file_system.read(&file_path).await
      {
        let tsconfig = tsconfig_cache.resolve(&file_path, &options.root_dir);

        let result = typescript::transpile(TransformerContext {
          content,
          path: file_path.clone(),
          source_url: req.uri.path().to_string(),
          source_maps: true,
          kind: kind.to_string(),
          root_dir: options.root_dir.clone(),
          tsconfig,
        });

        if let Ok(result) = result {
          for (key, value) in &options.custom_headers {
            res = res.header(key.as_str(), value.as_str());
          }

          res = res.header(headers::CONTENT_TYPE, JAVASCRIPT_CONTENT_TYPE);

          // Development mode emits source maps; they are inlined so the
          // response stays a single, self-contained request.
          let code = match result.map {
            Some(map) => format!(
              "{}\n//# sourceMappingURL=data:application/json;base64,{}\n",
              result.code,
              base64::engine::general_purpose::STANDARD.encode(map.as_bytes()),
            ),
            None => result.code,
          };

          let (body, mut writer) = ResponseBody::chunked(BODY_BUFFER_SIZE);

          tokio::task::spawn(async move {
            let _ = writer.write_all(code.as_bytes()).await;
            let _ = writer.shutdown().await;
          });

          return res.status(StatusCode::OK).body(body);
        }
      }

      let encoding = match encoding_resolver.as_ref() {
        Some(resolver) => resolver.resolve(&req),
        None => Encoding::Identity,
      };

      let cache_key = (file_path.clone(), encoding);

      // Fast path: replay a fully rendered response from the cache.
      if let Some(cache) = options.cache.as_ref()
        && let Some(cached) = cache.get(&cache_key)
      {
        for (key, value) in &options.custom_headers {
          res = res.header(key.as_str(), value.as_str());
        }

        res = res.header(headers::CONTENT_TYPE, cached.mime_type.as_str());

        if let Some(content_encoding) = cached.content_encoding {
          res = res.header(headers::CONTENT_ENCODING, content_encoding);
        }

        if let Some(etag) = cached.etag.as_ref() {
          if !has_modified(&req, etag) {
            return res.status(StatusCode::NOT_MODIFIED).empty();
          }
          res = res.header(headers::ETAG, etag.as_str());
        }

        let (body, mut writer) = ResponseBody::chunked(BODY_BUFFER_SIZE);

        tokio::task::spawn(async move {
          let _ = writer.write_all(cached.body.as_ref()).await;
          let _ = writer.shutdown().await;
        });

        return res.status(cached.status).body(body);
      }

      let mut status = StatusCode::OK;
      let mut mime_type = Option::<&String>::None;
      let mut cacheable = true;

      let mut file = match options.file_system.open(&file_path).await {
        Ok(file) => {
          mime_type.replace(&resolved.mime_type);
          file
        }
        Err(_) => match options.file_system.open(&fallback.path).await {
          Ok(file) => {
            status = fallback.status;
            mime_type.replace(&fallback.mime_type);
            cacheable = false;
            file
          }
          Err(_) => return res.status(StatusCode::NOT_FOUND).empty(),
        },
      };

      for (key, value) in &options.custom_headers {
        res = res.header(key.as_str(), value.as_str());
      }

      if let Some(mime_type) = mime_type.as_ref() {
        res = res.header(headers::CONTENT_TYPE, mime_type.as_str());
      }

      if let Some(content_encoding) = encoding.header_value() {
        res = res.header(headers::CONTENT_ENCODING, content_encoding);
      }

      let etag = match etag_resolver.as_ref() {
        Some(etag_resolver) => {
          etag_resolver
            .resolve(&mut *file, encoding.etag_suffix())
            .await?
        }
        None => None,
      };

      if let Some(etag) = etag.as_ref() {
        if !has_modified(&req, etag) {
          return res.status(StatusCode::NOT_MODIFIED).empty();
        }
        res = res.header(headers::ETAG, etag.as_str());
      }

      let cache_slot = match (options.cache.as_ref(), cacheable) {
        (Some(cache), true) => {
          let fits = match file.metadata().await {
            Ok(meta) => meta.len() as usize <= cache.max_entry_bytes(),
            Err(_) => false,
          };
          fits.then(|| (Arc::clone(cache), cache.generation()))
        }
        _ => None,
      };

      let (body, writer) = ResponseBody::chunked(BODY_BUFFER_SIZE);
      let mut tee = TeeWriter::new(
        writer,
        cache_slot
          .as_ref()
          .map(|(cache, _)| cache.max_entry_bytes()),
      );

      let content_encoding = encoding.header_value();
      let mime_type = mime_type.cloned();

      tokio::task::spawn(async move {
        let _ = match encoding {
          Encoding::Zstd => zstd_stream(&mut file, &mut tee).await,
          Encoding::Brotli => brotli_stream(&mut file, &mut tee).await,
          Encoding::Gzip => gzip_stream(&mut file, &mut tee).await,
          Encoding::Identity => uncompressed_stream(&mut file, &mut tee).await,
        };
        let _ = tee.shutdown().await;

        if let (Some((cache, generation)), Some(buffered)) = (cache_slot, tee.into_buffered()) {
          cache.set(
            cache_key,
            CachedResponse {
              status,
              mime_type: mime_type.unwrap_or_default(),
              content_encoding,
              etag,
              body: Arc::from(buffered),
            },
            generation,
          );
        }
      });

      res.status(status).body(body)
    })
  }))
}

fn has_modified(
  req: &Request,
  etag: &str,
) -> bool {
  if let Some(if_none_match) = req.headers.get(headers::IF_NONE_MATCH)
    && if_none_match == etag
  {
    return false;
  }
  true
}
