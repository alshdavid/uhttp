use std::future::Future;
use std::io;
use std::pin::Pin;

use tokio::io::AsyncReadExt;
use tokio::io::AsyncRead;
use tokio::net::tcp::OwnedReadHalf;

use crate::constants::HEADER_CONTENT_LENGTH;
use crate::Headers;

pub type BodyReaderRef = Box<dyn BodyReader>;

pub trait BodyReader: Send + Sync {
  fn read<'a>(&'a mut self, buf: &'a mut [u8]) -> Pin<Box<dyn Future<Output = io::Result<usize>> + Send + Sync + 'a>>;
}

pub struct Request {
  pub method: String,
  pub url: String,
  pub proto: String,
  pub headers: Headers,
  pub body: Box<dyn AsyncRead + Unpin + Send + Sync>,
  pub host: String,
}

impl std::fmt::Debug for Request {
  fn fmt(
    &self,
    f: &mut std::fmt::Formatter<'_>,
  ) -> std::fmt::Result {
    let content_length = match self.headers.get(HEADER_CONTENT_LENGTH) {
      Some(v) => v.to_owned(),
      None => format!("0"),
    };

    f.debug_struct("Request")
      .field("method", &self.method)
      .field("url", &self.url)
      .field("proto", &self.proto)
      .field("headers", &self.headers)
      .field("body", &format!("std::io::Read({} bytes)", content_length))
      .field("host", &self.host)
      .finish()
  }
}

pub struct HttpRequestReader {
  pub content_length: usize,
  pub cursor: usize,
  pub stream: Box<dyn AsyncRead>,
}

// impl BodyReader for HttpRequestReader {
//   fn read<'a>(
//     &'a mut self,
//     buf: &'a mut [u8],
//   ) -> Pin<Box<dyn Future<Output = io::Result<usize>> + Send + Sync + 'a>> {
//     Box::pin(async move {
//       if self.cursor >= self.content_length {
//         return Ok(0);
//       }
//       let count = self.stream.read(buf).await?;
//       self.cursor += count;
//       return Ok(count);
//     })
//   }
// }