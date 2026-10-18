use std::hash::DefaultHasher;
use std::hash::Hasher;
use std::io;
use std::io::SeekFrom;

use tokio::io::AsyncReadExt;
use tokio::io::AsyncSeekExt;

use super::super::file_server::BODY_BUFFER_SIZE;
use super::super::file_system::AsyncFile;

pub struct EtagHashResolver;

impl EtagHashResolver {
  pub async fn resolve(
    &self,
    file: &mut dyn AsyncFile,
    encoding: &str,
  ) -> io::Result<Option<String>> {
    let mut hasher = DefaultHasher::new();
    let mut buffer = [0u8; BODY_BUFFER_SIZE];

    loop {
      let n = file.read(&mut buffer).await?;
      if n == 0 {
        break;
      }
      hasher.write(&buffer[..n]);
    }

    file.seek(SeekFrom::Start(0)).await?;

    Ok(Some(format!("{:016x}{}", hasher.finish(), encoding)))
  }
}
