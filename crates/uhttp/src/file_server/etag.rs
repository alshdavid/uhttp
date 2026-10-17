use std::hash::DefaultHasher;
use std::hash::Hasher;

use tokio::io::AsyncReadExt;
use tokio::io::AsyncSeekExt;

use super::file_server::BODY_BUFFER_SIZE;
use super::file_server::ETagStrategy;
use super::file_system::AsyncFile;

pub(crate) async fn etag_file(
  file: &mut dyn AsyncFile,
  strategy: &ETagStrategy,
  encoding: &str,
) -> Result<Option<String>, std::io::Error> {
  match strategy {
    ETagStrategy::Hash => {
      let mut hasher = DefaultHasher::new();
      let mut buffer = [0u8; BODY_BUFFER_SIZE];

      loop {
        let n = file.read(&mut buffer).await?;
        if n == 0 {
          break;
        }
        hasher.write(&buffer[..n]);
      }

      file.seek(std::io::SeekFrom::Start(0)).await?;

      Ok(Some(format!("{:016x}{}", hasher.finish(), encoding)))
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
