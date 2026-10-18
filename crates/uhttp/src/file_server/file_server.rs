use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

use tokio::io::AsyncWriteExt;

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
use crate::HandleFunc;
use crate::Request;
use crate::ResponseBody;
use crate::StatusCode;
use crate::file_server::compression::uncompressed_stream;
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

  Ok(Box::new(move |req, mut res| {
    let options = Arc::clone(&options);
    let resolver = Arc::clone(&resolver);
    let etag_resolver = Arc::clone(&etag_resolver);
    let encoding_resolver = Arc::clone(&encoding_resolver);
    let fallback = Arc::clone(&fallback);

    Box::pin(async move {
      let resolved = resolver.resolve(req.uri.path()).await?;
      let file_path = resolved.path;

      let mut status = StatusCode::OK;
      let mut mime_type = Option::<&String>::None;

      let mut file = match options.file_system.open(&file_path).await {
        Ok(file) => {
          mime_type.replace(&resolved.mime_type);
          file
        }
        Err(_) => match options.file_system.open(&fallback.path).await {
          Ok(file) => {
            status = fallback.status;
            mime_type.replace(&fallback.mime_type);
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

      let encoding = match encoding_resolver.as_ref() {
        Some(resolver) => resolver.resolve(&req),
        None => Encoding::Identity,
      };

      if let Some(content_encoding) = encoding.header_value() {
        res = res.header(headers::CONTENT_ENCODING, content_encoding);
      }

      if let Some(etag_resolver) = etag_resolver.as_ref()
        && let Some(etag) = etag_resolver
          .resolve(&mut *file, encoding.etag_suffix())
          .await?
      {
        if !has_modified(&req, &etag) {
          return res.status(StatusCode::NOT_MODIFIED).empty();
        }
        res = res.header(headers::ETAG, etag.as_str());
      }

      let (body, mut writer) = ResponseBody::chunked(BODY_BUFFER_SIZE);

      tokio::task::spawn(async move {
        let _ = match encoding {
          Encoding::Zstd => zstd_stream(&mut file, &mut writer).await,
          Encoding::Brotli => brotli_stream(&mut file, &mut writer).await,
          Encoding::Gzip => gzip_stream(&mut file, &mut writer).await,
          Encoding::Identity => uncompressed_stream(&mut file, &mut writer).await,
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
