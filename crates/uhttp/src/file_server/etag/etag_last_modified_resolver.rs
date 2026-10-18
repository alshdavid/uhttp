use std::io;
use std::time::UNIX_EPOCH;

use super::super::file_system::AsyncFile;

pub struct EtagLastModifiedResolver;

impl EtagLastModifiedResolver {
  pub async fn resolve(
    &self,
    file: &mut dyn AsyncFile,
    encoding: &str,
  ) -> io::Result<Option<String>> {
    let meta = file.metadata().await?;
    let etag = format!(
      "{:x}{:x}{}",
      meta
        .modified()?
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos(),
      meta.len(),
      encoding,
    );

    Ok(Some(etag))
  }
}
