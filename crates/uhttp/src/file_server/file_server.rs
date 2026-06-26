use std::path::PathBuf;
use std::sync::Arc;

use tokio::io::AsyncRead;
use tokio::io::AsyncReadExt;
use tokio::io::AsyncSeekExt;
use tokio::io::AsyncWrite;
use tokio::io::BufReader;
use xxhash_rust::xxh3::Xxh3;

use crate::HandleFunc;
use crate::Request;
use crate::StatusCode;

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
  /// Relative path to fallback URL. Defaults to "404.html"
  pub fallback_route: Option<String>,
  /// Defaults to [`crate::StatusCode::NOT_FOUND] (404)
  pub fallback_status: Option<StatusCode>,
}

/// Serve files from the filesystem
pub fn create(options: FileServerOptions) -> HandleFunc {
  let options = Arc::new(options);

  Box::new(move |req, mut res| {
    let options = Arc::clone(&options);

    Box::pin(async move {
      let url_path = determine_file(req.uri().path());
      let mut full_path = options.dir.join(&url_path);
      let mut fallback_status = None::<StatusCode>;

      let mut file = match tokio::fs::File::open(&full_path).await {
        Ok(file) => file,
        Err(_) => {
          let fallback_path = match options.fallback_route.as_ref() {
            Some(path) => options.dir.join(path),
            None => options.dir.join("404.html"),
          };
          fallback_status = Some(options.fallback_status.unwrap_or(StatusCode::NOT_FOUND));
          match tokio::fs::File::open(&fallback_path).await {
            Ok(file) => {
              full_path = fallback_path;
              file
            }
            Err(_) => {
              res.write_head(StatusCode::NOT_FOUND).await?;
              return Ok(());
            }
          }
        }
      };

      let mime_type = mime_guess::from_path(&full_path)
        .first_or_octet_stream()
        .to_string();

      res.header().add("Content-Type", &mime_type).await?;

      if options.compress
        && let Some(accept_encoding) = req.headers().get("Accept-Encoding")
        && let Ok(accept_encoding) = accept_encoding.to_str()
      {
        if accept_encoding.contains("zstd") {
          res.header().add("Content-Encoding", "zstd").await?;

          if let Some(etag) = etag_file(&mut file, &options.etag, "zstd").await? {
            if !has_modified(&req, &etag) {
              res.write_head(StatusCode::NOT_MODIFIED).await?;
              return Ok(());
            }
            res.header().add("ETag", &etag).await?;
          }
          match fallback_status {
            Some(status) => res.write_head(status).await?,
            None => res.write_head(StatusCode::OK).await?,
          };
          zstd_stream(&mut file, &mut res).await?;
          return Ok(());
        } else if accept_encoding.contains("br") {
          res.header().add("Content-Encoding", "br").await?;

          if let Some(etag) = etag_file(&mut file, &options.etag, "br").await? {
            if !has_modified(&req, &etag) {
              res.write_head(StatusCode::NOT_MODIFIED).await?;
              return Ok(());
            }
            res.header().add("ETag", &etag).await?;
          }
          match fallback_status {
            Some(status) => res.write_head(status).await?,
            None => res.write_head(StatusCode::OK).await?,
          };
          brotli_stream(&mut file, &mut res).await?;
          return Ok(());
        } else if accept_encoding.contains("gz") {
          res.header().add("Content-Encoding", "gzip").await?;
          if let Some(etag) = etag_file(&mut file, &options.etag, "gzip").await? {
            if !has_modified(&req, &etag) {
              res.write_head(StatusCode::NOT_MODIFIED).await?;
              return Ok(());
            }
            res.header().add("ETag", &etag).await?;
          }
          match fallback_status {
            Some(status) => res.write_head(status).await?,
            None => res.write_head(StatusCode::OK).await?,
          };
          gzip_stream(&mut file, &mut res).await?;
          return Ok(());
        }
      }

      if let Some(etag) = etag_file(&mut file, &options.etag, "").await? {
        if !has_modified(&req, &etag) {
          res.write_head(StatusCode::NOT_MODIFIED).await?;
          return Ok(());
        }
        res.header().add("ETag", &etag).await?;
      }

      match fallback_status {
        Some(status) => res.write_head(status).await?,
        None => res.write_head(StatusCode::OK).await?,
      };
      tokio::io::copy(&mut file, &mut res).await?;
      Ok(())
    })
  })
}

fn has_modified(
  req: &Request,
  etag: &str,
) -> bool {
  if let Some(if_none_match) = req.headers().get("If-None-Match")
    && if_none_match == etag
  {
    return false;
  }
  true
}

fn determine_file(input: &str) -> PathBuf {
  if input == "/" {
    PathBuf::from("/index.html".trim_start_matches("/"))
  } else if PathBuf::from(input).extension().is_some() {
    PathBuf::from(input.trim_start_matches("/"))
  } else {
    PathBuf::from(format!("{}.html", input.trim_start_matches("/")))
  }
}

async fn etag_file(
  file: &mut tokio::fs::File,
  strategy: &ETagStrategy,
  encoding: &str,
) -> Result<Option<String>, std::io::Error> {
  match strategy {
    ETagStrategy::Hash => {
      let file_handle_copy = file.try_clone().await?;
      let mut reader = BufReader::new(file_handle_copy);
      let mut hasher = Xxh3::new();
      let mut buffer = [0u8; 64 * 1024];

      loop {
        let n = reader.read(&mut buffer).await?;
        if n == 0 {
          break;
        }
        hasher.update(&buffer[..n]);
      }

      file.seek(std::io::SeekFrom::Start(0)).await?;

      Ok(Some(format!("{:016x}", hasher.digest())))
    }
    ETagStrategy::LastModified => {
      let meta = file.metadata().await?;
      let etag = format!(
        "{:x}{:x}{}",
        meta
          .modified()?
          .duration_since(std::time::UNIX_EPOCH)
          .unwrap()
          .as_nanos(),
        meta.len(),
        encoding,
      );
      Ok(Some(etag))
    }
    ETagStrategy::Disabled => Ok(None),
  }
}

async fn gzip_stream<R, W>(
  input: R,
  output: &mut W,
) -> Result<u64, std::io::Error>
where
  R: AsyncRead + Unpin,
  W: AsyncWrite + Unpin,
{
  use async_compression::tokio::bufread::GzipEncoder;
  let mut encoder = GzipEncoder::new(BufReader::new(input));
  tokio::io::copy(&mut encoder, output).await
}

async fn brotli_stream<R, W>(
  input: R,
  output: &mut W,
) -> Result<u64, std::io::Error>
where
  R: AsyncRead + Unpin,
  W: AsyncWrite + Unpin,
{
  use async_compression::tokio::bufread::BrotliEncoder;
  let mut encoder = BrotliEncoder::new(BufReader::new(input));
  tokio::io::copy(&mut encoder, output).await
}

async fn zstd_stream<R, W>(
  input: R,
  output: &mut W,
) -> Result<u64, std::io::Error>
where
  R: AsyncRead + Unpin,
  W: AsyncWrite + Unpin,
{
  use async_compression::tokio::bufread::ZstdEncoder;
  let mut encoder = ZstdEncoder::new(BufReader::new(input));
  tokio::io::copy(&mut encoder, output).await
}
