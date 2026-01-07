use std::io;

use tokio::io::AsyncRead;
use tokio::io::AsyncReadExt;

use crate::constants::DEFAULT_BUFFER_SIZE;

pub async fn bytes(reader: &mut (impl AsyncRead + Unpin)) -> io::Result<Vec<u8>> {
  let mut body = Vec::<u8>::new();
  let mut buf = vec![0u8; DEFAULT_BUFFER_SIZE];

  loop {
    let count = reader.read(&mut buf).await?;
    body.extend(buf.drain(..count));
    if count == 0 {
      break;
    }
  }
  Ok(body)
}
