use std::io::Read;
use std::net::TcpStream;

use crate::constants::HEADER_CONTENT_LENGTH;
use crate::Headers;

pub struct Request {
  pub method: String,
  pub url: String,
  pub proto: String,
  pub headers: Headers,
  pub body: Box<dyn Read>,
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
  pub stream: TcpStream,
}

impl Read for HttpRequestReader {
  fn read(
    &mut self,
    buf: &mut [u8],
  ) -> std::io::Result<usize> {
    if self.content_length == 0 {
      return Ok(0);
    }
    let count: usize = self.stream.read(buf)?;
    if count == 0 {
      return Ok(0);
    }
    self.cursor += count;
    if self.cursor >= self.content_length {
      return Ok(0);
    }
    return Ok(count);
  }
}
