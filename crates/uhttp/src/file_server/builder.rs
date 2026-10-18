use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

use super::ETagStrategy;
use super::FileServerCache;
use super::FileServerOptions;
use super::create;
use super::file_system::AsyncFileSystem;
use super::file_system::TokioFileSystem;
use crate::HandleFunc;
use crate::StatusCode;
#[cfg(feature = "file_server_transpile")]
use crate::file_server::TranspileOptions;

/// Serve files from the filesystem.
///
/// Start with [`FileServer::builder`], configure the handler, then finish with
/// [`FileServerBuilder::build`].
///
/// ```rust
/// # fn main() -> uhttp::Result<()> {
/// use uhttp::file_server::FileServer;
///
/// let handler = FileServer::builder()
///   .root_dir("./static")
///   .compress(true)
///   .build()?;
/// # Ok(())
/// # }
/// ```
pub struct FileServer;

impl FileServer {
  /// Create a [`FileServerBuilder`] to configure and construct a file server
  pub fn builder() -> FileServerBuilder {
    FileServerBuilder::default()
  }
}

/// Builder for a [`FileServer`] handler.
///
/// Construct one with [`FileServer::builder`], configure it with the fluent
/// methods, and finish with [`FileServerBuilder::build`].
pub struct FileServerBuilder {
  file_system: Arc<dyn AsyncFileSystem>,
  root_dir: PathBuf,
  compress: bool,
  etag: ETagStrategy,
  custom_headers: HashMap<String, String>,
  cache: Option<Arc<FileServerCache>>,
  fallback_route: Option<String>,
  fallback_status: Option<StatusCode>,
  implicit_extensions: bool,
  #[cfg(feature = "file_server_transpile")]
  transpile: Option<TranspileOptions>,
}

impl Default for FileServerBuilder {
  fn default() -> Self {
    Self {
      file_system: TokioFileSystem::new(),
      root_dir: PathBuf::from("."),
      compress: false,
      etag: ETagStrategy::default(),
      custom_headers: HashMap::new(),
      cache: None,
      fallback_route: None,
      fallback_status: None,
      implicit_extensions: false,
      #[cfg(feature = "file_server_transpile")]
      transpile: None,
    }
  }
}

impl FileServerBuilder {
  /// The filesystem files are read from. Defaults to [`TokioFileSystem`].
  pub fn file_system(
    mut self,
    file_system: Arc<dyn AsyncFileSystem>,
  ) -> Self {
    self.file_system = file_system;
    self
  }

  /// The root directory files are served from. Defaults to the current dir.
  pub fn root_dir(
    mut self,
    root_dir: impl Into<PathBuf>,
  ) -> Self {
    self.root_dir = root_dir.into();
    self
  }

  /// Send back compressed responses. Defaults to `false`.
  pub fn compress(
    mut self,
    compress: bool,
  ) -> Self {
    self.compress = compress;
    self
  }

  /// How etags are supplied. Defaults to [`ETagStrategy::LastModified`].
  pub fn etag(
    mut self,
    etag: ETagStrategy,
  ) -> Self {
    self.etag = etag;
    self
  }

  /// Add a single custom response header
  pub fn header(
    mut self,
    key: impl Into<String>,
    value: impl Into<String>,
  ) -> Self {
    self.custom_headers.insert(key.into(), value.into());
    self
  }

  /// Replace the set of custom response headers
  pub fn custom_headers(
    mut self,
    custom_headers: HashMap<String, String>,
  ) -> Self {
    self.custom_headers = custom_headers;
    self
  }

  /// Enable the in-memory response cache
  pub fn cache(
    mut self,
    cache: Arc<FileServerCache>,
  ) -> Self {
    self.cache = Some(cache);
    self
  }

  /// Relative path to the fallback response. Defaults to `"404.html"`.
  pub fn fallback_route(
    mut self,
    fallback_route: impl Into<String>,
  ) -> Self {
    self.fallback_route = Some(fallback_route.into());
    self
  }

  /// Status code sent with the fallback response.
  ///
  /// Defaults to [`StatusCode::NOT_FOUND`] (404).
  pub fn fallback_status(
    mut self,
    fallback_status: StatusCode,
  ) -> Self {
    self.fallback_status = Some(fallback_status);
    self
  }

  /// Try to resolve HTML files on paths without a specified file.
  ///
  /// `/`    -> `/index.html`
  /// `/foo` -> `/foo/index.html` | `/foo.html`
  pub fn implicit_extensions(
    mut self,
    implicit_extensions: bool,
  ) -> Self {
    self.implicit_extensions = implicit_extensions;
    self
  }

  #[cfg(feature = "file_server_transpile")]
  /// JIT-transpile TypeScript and SCSS sources on request.
  ///
  /// Requires the `file_server_transpile` feature.
  pub fn transpile(
    mut self,
    options: TranspileOptions,
  ) -> Self {
    self.transpile = Some(options);
    self
  }

  /// Construct the [`FileServer`] handler
  pub fn build(self) -> crate::Result<HandleFunc> {
    create(FileServerOptions {
      file_system: self.file_system,
      root_dir: self.root_dir,
      compress: self.compress,
      etag: self.etag,
      custom_headers: self.custom_headers,
      cache: self.cache,
      fallback_route: self.fallback_route,
      fallback_status: self.fallback_status,
      implicit_extensions: self.implicit_extensions,
      #[cfg(feature = "file_server_transpile")]
      transpile: self.transpile,
    })
  }
}
