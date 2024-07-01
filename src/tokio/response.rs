use std::future::Future;
use std::io;
use std::pin::Pin;

use tokio::io::AsyncWriteExt;
use tokio::net::tcp::OwnedWriteHalf;

use crate::Headers;

pub type ResponseRef = Box<dyn Response>;

pub trait Response: Send + Sync {
  fn headers(&mut self) -> &mut Headers;
  fn write_header<'a>(
    &'a mut self,
    status_code: usize,
  ) -> Pin<Box<dyn Future<Output = io::Result<()>> + Send + Sync + 'a>>;
  fn write<'a>(
    &'a mut self,
    buf: &'a [u8],
  ) -> Pin<Box<dyn Future<Output = io::Result<usize>> + Send + Sync + 'a>>;
  fn flush<'a>(&'a mut self) -> Pin<Box<dyn Future<Output = io::Result<()>> + 'a>>;
}

pub struct HttpResponse {
  pub(super) headers: Headers,
  pub(super) stream: OwnedWriteHalf,
}

impl Response for HttpResponse {
  fn headers(&mut self) -> &mut Headers {
    &mut self.headers
  }

  fn write<'a>(
    &'a mut self,
    buf: &'a [u8],
  ) -> Pin<Box<dyn Future<Output = io::Result<usize>> + Send + Sync + 'a>> {
    Box::pin(async move { self.stream.write(buf).await })
  }

  fn flush<'a>(&'a mut self) -> Pin<Box<dyn Future<Output = io::Result<()>> + 'a>> {
    Box::pin(async move { self.stream.flush().await })
  }

  fn write_header<'a>(
    &'a mut self,
    status_code: usize,
  ) -> Pin<Box<dyn Future<Output = io::Result<()>> + Send + Sync + 'a>> {
    Box::pin(async move {
      let response_code = format!("{}", status_code);
      let response_message = "OK";

      // Write Reply
      self.stream.write(b"HTTP/1.1 ").await?;
      // Status Code
      self.stream.write(response_code.as_bytes()).await?;
      self.stream.write(b" ").await?;

      // Message
      self.stream.write(response_message.as_bytes()).await?;
      self.stream.write(b"\r\n").await?;

      for (key, values) in self.headers.iter() {
        self.stream.write(key.as_bytes()).await?;
        self.stream.write(b": ").await?;

        self.stream.write(values.as_bytes()).await?;
        self.stream.write(b"\r\n").await?;
      }

      self.stream.write(b"\r\n").await?;
      self.stream.flush().await?;

      Ok(())
    })
  }
}
