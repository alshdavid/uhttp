use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

use dyn_fs::ext::PathExt;
use mime_guess::Mime;
use tokio::io::AsyncWriteExt;

use super::compression::brotli_stream;
use super::compression::gzip_stream;
use super::compression::zstd_stream;
use super::encoding::Encoding;
use super::encoding::negotiate_encoding;
use super::etag::etag_file;
use super::fallback_file::FallbackFile;
use super::fallback_file::FallbackFileResolveOptions;
use super::file_system::AsyncFileSystem;
use super::mime::detect_mime_for_path;
use super::resolver::ResolveImplicitExtensionOptions;
use super::resolver::resolve_implicit_extension;
use crate::HandleFunc;
use crate::Request;
use crate::ResponseBody;
use crate::StatusCode;
use crate::headers;

/// Size of the buffer bodies are streamed through on their way to the client
pub const BODY_BUFFER_SIZE: usize = 64 * 1024;

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
  /// Relative path to fallback URL. Defaults to "404.html"
  pub fallback_route: Option<String>,
  /// Defaults to [`crate::StatusCode::NOT_FOUND] (404)
  pub fallback_status: Option<StatusCode>,
  /// Try to resolve html files on paths without specified files
  ///  `/`    -> `/index.html`
  ///  `/foo` -> `/foo/index.html` | `/foo.html`
  pub implicit_extensions: bool,
}

/// Serve files from the filesystem
pub fn create(options: FileServerOptions) -> crate::Result<HandleFunc> {
  let options = Arc::new(options);

  let fallback = Arc::new(FallbackFile::resolve(FallbackFileResolveOptions {
    root_dir: &options.root_dir,
    fallback_route: &options.fallback_route,
    fallback_status: &options.fallback_status,
  }));

  Ok(Box::new(move |req, mut res| {
    let options = Arc::clone(&options);
    let fallback = Arc::clone(&fallback);

    Box::pin(async move {
      let url_path = match options.implicit_extensions {
        true => {
          resolve_implicit_extension(ResolveImplicitExtensionOptions {
            fs: &*options.file_system,
            root_dir: &options.root_dir,
            input: req.uri.path(),
          })
          .await?
        }
        false => PathBuf::from(req.uri.path()),
      };

      let file_path = options.root_dir.join(&url_path);

      let mut status = StatusCode::OK;
      let mut mime_type = Option::<Mime>::None;

      for (key, value) in &options.custom_headers {
        res = res.header(key.as_str(), value.as_str());
      }

      let mut file = match options.file_system.open(&file_path).await {
        Ok(file) => {
          mime_type.replace(detect_mime_for_path(&url_path));
          file
        }
        Err(_) => match options.file_system.open(&fallback.path).await {
          Ok(file) => {
            status = fallback.status;
            mime_type.replace(fallback.mime_type.clone());
            file
          }
          Err(_) => return res.status(StatusCode::NOT_FOUND).empty(),
        },
      };

      if let Some(mime_type) = mime_type.as_ref() {
        res = res.header(headers::CONTENT_TYPE, mime_type.to_string());
      }

      let encoding = negotiate_encoding(&req, options.compress);

      if let Some(content_encoding) = encoding.header_value() {
        res = res.header(headers::CONTENT_ENCODING, content_encoding);
      }

      if let Some(etag) = etag_file(&mut *file, &options.etag, encoding.etag_suffix()).await? {
        if !has_modified(&req, &etag) {
          return res.status(StatusCode::NOT_MODIFIED).empty();
        }
        res = res.header(headers::ETAG, etag.as_str());
      }

      let (body, mut writer) = ResponseBody::chunked(BODY_BUFFER_SIZE);

      // The body is written after the response head has been handed back, so the
      // copy has to outlive this future. A client hanging up mid transfer shows up
      // as a write error here and there is no longer anywhere to report it to.
      tokio::task::spawn(async move {
        let _ = match encoding {
          Encoding::Zstd => zstd_stream(&mut file, &mut writer).await,
          Encoding::Brotli => brotli_stream(&mut file, &mut writer).await,
          Encoding::Gzip => gzip_stream(&mut file, &mut writer).await,
          Encoding::Identity => tokio::io::copy(&mut file, &mut writer).await,
        };
        let _ = writer.shutdown().await;
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
