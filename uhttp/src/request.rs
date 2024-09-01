use std::io;

use tokio::sync::mpsc::Receiver;

use crate::Headers;

#[async_trait::async_trait]
pub trait HttpReader: Send + Sync {
  async fn read(&mut self, buf: &mut [u8]) -> io::Result<usize>;
}

pub struct Request {
  pub method: String,
  pub url: String,
  pub proto: String,
  pub headers: Headers,
  pub (crate) body_buf: Vec<u8>,
  pub (crate) body: Receiver<Vec<u8>>,
  pub host: String,
}

impl std::fmt::Debug for Request {
  fn fmt(
    &self,
    f: &mut std::fmt::Formatter<'_>,
  ) -> std::fmt::Result {
    f.debug_struct("Request")
      .field("method", &self.method)
      .field("url", &self.url)
      .field("proto", &self.proto)
      .field("headers", &self.headers)
      // .field("body", &format!("std::io::Read({} bytes)", content_length))
      .field("host", &self.host)
      .finish()
  }
}

#[async_trait::async_trait]
impl HttpReader for Request {
  async fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
    let inner_len = self.body_buf.len();
    let buf_len = buf.len();
    let mut cursor = 0;

    if inner_len != 0 {
      cursor = transfer(&mut self.body_buf, buf, 0);
    }

    if cursor >= buf_len {
      return Ok(cursor);
    }

    if let Some(mut bytes) = self.body.recv().await {
      if bytes.len() == 0 {
        return Ok(cursor);
      }
      cursor = transfer(&mut bytes, buf, cursor);
      self.body_buf = bytes;
      return Ok(cursor);
    }

    return Ok(cursor);
  }
}

pub fn transfer(
  src: &mut Vec<u8>,
  dest: &mut [u8],
  start: usize,
) -> usize {
  let src_len = src.len();
  let dest_len = dest.len();
  let mut available = dest_len - start;

  if available > src_len {
    available = src_len
  }

  for (i, byte) in src.drain(..available).enumerate() {
    dest[i + start] = byte;
  }

  return available;
}
