use std::io::{self};
use std::str::FromStr;

use http::HeaderMap;
use http::HeaderName;
use http::HeaderValue;
use tokio::io::AsyncWrite;
use tokio::sync::mpsc::UnboundedSender;

use crate::Response;

pub struct Http1Response {
  pub(super) tx_writer: UnboundedSender<Http1ResponseTx>,
  pub(super) headers: HeaderMap,
}

pub enum Http1ResponseTx {
  Write(Vec<u8>),
  Flush,
  Shutdown,
}

impl Response for Http1Response {
  fn headers_ref(&self) -> &HeaderMap {
    &self.headers
  }

  fn headers_mut(&mut self) -> &mut HeaderMap {
    &mut self.headers
  }

  fn split_writer(&mut self) -> Box<dyn AsyncWrite + Send + Sync + Unpin> {
    Box::new(Http1AsyncWriter {
      tx_writer: self.tx_writer.clone(),
    })
  }

  fn set_header(
    &mut self,
    key: &str,
    value: &str,
  ) {
    let key = HeaderName::from_str(key).unwrap();
    let value = HeaderValue::from_str(value).unwrap();
    self.headers.append(key, value);
  }
}

impl AsyncWrite for Http1Response {
  fn poll_write(
    self: std::pin::Pin<&mut Self>,
    _cx: &mut std::task::Context<'_>,
    buf: &[u8],
  ) -> std::task::Poll<Result<usize, io::Error>> {
    let len = buf.len();
    if let Err(_error) = self.tx_writer.send(Http1ResponseTx::Write(buf.to_vec())) {
      return std::task::Poll::Ready(Err(io::Error::other("Failed to write bytes")));
    };
    std::task::Poll::Ready(Ok(len))
  }

  fn poll_flush(
    self: std::pin::Pin<&mut Self>,
    _cx: &mut std::task::Context<'_>,
  ) -> std::task::Poll<Result<(), io::Error>> {
    if let Err(_error) = self.tx_writer.send(Http1ResponseTx::Flush) {
      return std::task::Poll::Ready(Err(io::Error::other("Failed to flush bytes")));
    };
    std::task::Poll::Ready(Ok(()))
  }

  fn poll_shutdown(
    self: std::pin::Pin<&mut Self>,
    _cx: &mut std::task::Context<'_>,
  ) -> std::task::Poll<Result<(), io::Error>> {
    if let Err(_error) = self.tx_writer.send(Http1ResponseTx::Shutdown) {
      return std::task::Poll::Ready(Err(io::Error::other("Failed to shutdown")));
    };
    std::task::Poll::Ready(Ok(()))
  }
}

pub struct Http1AsyncWriter {
  pub(super) tx_writer: UnboundedSender<Http1ResponseTx>,
}

impl AsyncWrite for Http1AsyncWriter {
  fn poll_write(
    self: std::pin::Pin<&mut Self>,
    _cx: &mut std::task::Context<'_>,
    buf: &[u8],
  ) -> std::task::Poll<Result<usize, io::Error>> {
    let len = buf.len();
    if let Err(_error) = self.tx_writer.send(Http1ResponseTx::Write(buf.to_vec())) {
      return std::task::Poll::Ready(Err(io::Error::other("Failed to write bytes")));
    };
    std::task::Poll::Ready(Ok(len))
  }

  fn poll_flush(
    self: std::pin::Pin<&mut Self>,
    _cx: &mut std::task::Context<'_>,
  ) -> std::task::Poll<Result<(), io::Error>> {
    if let Err(_error) = self.tx_writer.send(Http1ResponseTx::Flush) {
      return std::task::Poll::Ready(Err(io::Error::other("Failed to flush bytes")));
    };
    std::task::Poll::Ready(Ok(()))
  }

  fn poll_shutdown(
    self: std::pin::Pin<&mut Self>,
    _cx: &mut std::task::Context<'_>,
  ) -> std::task::Poll<Result<(), io::Error>> {
    if let Err(_error) = self.tx_writer.send(Http1ResponseTx::Shutdown) {
      return std::task::Poll::Ready(Err(io::Error::other("Failed to shutdown")));
    };
    std::task::Poll::Ready(Ok(()))
  }
}
