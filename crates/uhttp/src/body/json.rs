use std::io;

use serde::de::DeserializeOwned;
use tokio::io::AsyncRead;

use super::bytes::bytes;

pub async fn json<T: DeserializeOwned>(reader: &mut (impl AsyncRead + Unpin)) -> io::Result<T> {
  let body = bytes(reader).await?;
  match serde_json::from_slice(&body) {
    Ok(v) => Ok(v),
    Err(err) => Err(io::Error::other(format!("{}", err))),
  }
}
