use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

use include_dir::Dir;
use tokio::io::AsyncWriteExt;

use crate::HandleFunc;
use crate::Request;
use crate::StatusCode;

struct CacheEntry {
  mime_type: String,
  contents: Vec<u8>,
  contents_br: Vec<u8>,
  contents_gzip: Vec<u8>,
  hash: String,
  hash_br: String,
  hash_gzip: String,
}

impl std::fmt::Debug for CacheEntry {
  fn fmt(
    &self,
    f: &mut std::fmt::Formatter<'_>,
  ) -> std::fmt::Result {
    f.debug_struct("CacheEntry")
      .field("mime_type", &self.mime_type)
      .field("contents", &self.contents.len())
      .field("contents_br", &self.contents_br.len())
      .field("contents_gzip", &self.contents_gzip.len())
      .field("hash", &self.hash)
      .field("hash_br", &self.hash_br)
      .field("hash_gzip", &self.hash_gzip)
      .finish()
  }
}

#[derive(Debug)]
pub struct FileServerIncludeDirOptions {
  /// The root directory to get files from
  pub dir: include_dir::Dir<'static>,
  /// Send back compressed responses
  pub compress: bool,
}

/// Serve embedded files using [`include_dir`]
pub fn create_include_dir(options: FileServerIncludeDirOptions) -> HandleFunc {
  let options = Arc::new(options);
  let mut cache = HashMap::<PathBuf, CacheEntry>::new();

  // Precompute contents
  for file in get_all_files(&options.dir) {
    let contents = file.contents().to_vec();
    let contents_hash = hash(&contents);

    let contents_br = if options.compress {
      brotli(&contents)
    } else {
      Vec::new()
    };
    let contents_br_hash = if options.compress {
      hash(&contents_br)
    } else {
      String::default()
    };

    let contents_gzip = if options.compress {
      gzip(&contents)
    } else {
      Vec::new()
    };
    let contents_gzip_hash = if options.compress {
      hash(&contents_gzip)
    } else {
      String::default()
    };

    let mime_type = mime_guess::from_path(file.path())
      .first_or_octet_stream()
      .to_string();

    let entry = CacheEntry {
      mime_type,
      contents,
      contents_br,
      contents_gzip,
      hash: contents_hash,
      hash_br: contents_br_hash,
      hash_gzip: contents_gzip_hash,
    };

    cache.insert(file.path().to_path_buf(), entry);
  }

  let cache = Arc::new(cache);

  Box::new(move |req, mut res| {
    let options = Arc::clone(&options);
    let cache = Arc::clone(&cache);

    Box::pin(async move {
      let url_path = determine_file(req.uri().path());

      let Some(file) = cache.get(&url_path) else {
        res.write_head(StatusCode::NOT_FOUND).await?;
        return Ok(());
      };

      res.header().add("Content-Type", &file.mime_type).await?;

      if options.compress
        && let Some(accept_encoding) = get_header(&req, "Accept-Encoding")
      {
        if accept_encoding.contains("br") {
          res.header().add("Content-Encoding", "br").await?;

          if !has_modified(&req, &file.hash) {
            res.write_head(StatusCode::NOT_MODIFIED).await?;
            return Ok(());
          }
          res.header().add("ETag", &file.hash_br).await?;
          res.write_all(&file.contents_br).await?;
          res.write_head(StatusCode::OK).await?;
          return Ok(());
        } else if accept_encoding.contains("gz") {
          res.header().add("Content-Encoding", "gzip").await?;

          if !has_modified(&req, &file.hash) {
            res.write_head(StatusCode::NOT_MODIFIED).await?;
            return Ok(());
          }
          res.header().add("ETag", &file.hash_gzip).await?;
          res.write_all(&file.contents_gzip).await?;
          res.write_head(StatusCode::OK).await?;
          return Ok(());
        }
      }

      if !has_modified(&req, &file.hash) {
        res.write_head(StatusCode::NOT_MODIFIED).await?;
        return Ok(());
      }

      res.header().add("ETag", &file.hash).await?;
      res.write_all(&file.contents).await?;
      res.write_head(StatusCode::OK).await?;
      Ok(())
    })
  })
}

fn get_header<'a>(
  req: &'a Request,
  name: &str,
) -> Option<&'a str> {
  let header = req.headers().get(name)?;

  let Ok(header) = header.to_str() else {
    return None;
  };

  Some(header)
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

fn hash(input: &[u8]) -> String {
  let mut hasher = xxhash_rust::xxh3::Xxh3::new();
  hasher.update(input);
  format!("{:016x}", hasher.digest())
}

fn brotli(input: &[u8]) -> Vec<u8> {
  use std::io::Write;

  use brotli;

  let mut writer = brotli::CompressorWriter::new(Vec::new(), 4096, 11, 22);
  writer.write_all(input).unwrap();
  writer.into_inner()
}

fn gzip(input: &[u8]) -> Vec<u8> {
  use std::io::Write;

  use flate2::Compression;
  use flate2::write::GzEncoder;

  let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
  encoder
    .write_all(input)
    .expect("Failed to write to Gzip encoder");
  encoder.finish().expect("Failed to finish Gzip encoding")
}

fn get_all_files(dir: &Dir<'static>) -> Vec<include_dir::File<'static>> {
  let mut files = Vec::new();
  collect_files_recursive(dir, &mut files);
  files
}

fn collect_files_recursive(
  dir: &Dir<'static>,
  list: &mut Vec<include_dir::File<'static>>,
) {
  for file in dir.files() {
    list.push(file.clone());
  }

  for subdir in dir.dirs() {
    collect_files_recursive(subdir, list);
  }
}
