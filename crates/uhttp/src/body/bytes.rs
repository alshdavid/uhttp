use std::io;

use tokio::io::AsyncRead;
use tokio::io::AsyncReadExt;

pub async fn bytes(reader: &mut (impl AsyncRead + Unpin)) -> io::Result<Vec<u8>> {
  let mut body = Vec::new();
  reader.read_to_end(&mut body).await?;
  Ok(body)
}
