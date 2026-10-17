use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

use tokio::io::AsyncWriteExt;

use super::compression::brotli_stream;
use super::compression::gzip_stream;
use super::compression::zstd_stream;
use super::encoding::Encoding;
use super::encoding::negotiate_encoding;
use super::etag::etag_file;
use super::resolver::determine_file;
use super::resolver::try_extension;
use crate::HandleFunc;
use crate::Request;
use crate::ResponseBody;
use crate::StatusCode;

/// Size of the buffer bodies are streamed through on their way to the client
pub(crate) const BODY_BUFFER_SIZE: usize = 64 * 1024;

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

#[derive(Debug)]
pub struct FileServerOptions {
  /// The root directory to get files from
  pub dir: PathBuf,
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
}

/// Serve files from the filesystem
pub fn create(options: FileServerOptions) -> crate::Result<HandleFunc> {
  let options = Arc::new(options);

  let fallback_path = Arc::new(match options.fallback_route.as_ref() {
    Some(path) => options.dir.join(path),
    None => options.dir.join("404.html"),
  });

  let fallback_status = match options.fallback_status.as_ref() {
    Some(status) => *status,
    None => StatusCode::NOT_FOUND,
  };

  let fallback_ext = Arc::new(try_extension(&fallback_path)?.to_string());

  Ok(Box::new(move |req, mut res| {
    let options = Arc::clone(&options);
    let fallback_path = Arc::clone(&fallback_path);
    let fallback_ext = Arc::clone(&fallback_ext);

    Box::pin(async move {
      let url_path = determine_file(req.uri.path());
      let mut extension = try_extension(&url_path)?;
      let mut status = StatusCode::OK;

      for (key, value) in &options.custom_headers {
        res = res.header(key.as_str(), value.as_str());
      }

      let mut file = match tokio::fs::File::open(&options.dir.join(&url_path)).await {
        Ok(file) => file,
        Err(_) => match tokio::fs::File::open(&*fallback_path).await {
          Ok(file) => {
            status = fallback_status;
            extension = &fallback_ext;
            file
          }
          Err(_) => return res.status(StatusCode::NOT_FOUND).body(""),
        },
      };

      let mime_type = mime_guess::from_ext(extension)
        .first_or_octet_stream()
        .to_string();

      res = res.header("Content-Type", mime_type.as_str());

      let encoding = negotiate_encoding(&req, options.compress);

      if let Some(content_encoding) = encoding.header_value() {
        res = res.header("Content-Encoding", content_encoding);
      }

      if let Some(etag) = etag_file(&mut file, &options.etag, encoding.etag_suffix()).await? {
        if !has_modified(&req, &etag) {
          return res.status(StatusCode::NOT_MODIFIED).body("");
        }
        res = res.header("ETag", etag.as_str());
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
  if let Some(if_none_match) = req.headers.get("If-None-Match")
    && if_none_match == etag
  {
    return false;
  }
  true
}
