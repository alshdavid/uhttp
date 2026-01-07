use std::io;

use tokio::io::AsyncRead;

use super::bytes::bytes;

pub async fn utf8(reader: &mut (impl AsyncRead + Unpin)) -> io::Result<String> {
  let body = bytes(reader).await?;
  String::from_utf8(body).map_err(|err| io::Error::other(err))
}
