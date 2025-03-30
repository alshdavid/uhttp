use std::io::Write;
use std::io::{self};
use std::str::FromStr;

use http::HeaderMap;
use http::HeaderName;
use http::HeaderValue;
use tokio::io::AsyncWrite;
use tokio::sync::mpsc::UnboundedSender;

use crate::Response;

pub struct Http1Response {
  pub(super) tx_writer: UnboundedSender<Vec<u8>>,
  pub(super) headers: HeaderMap,
}

impl Write for Http1Response {
  fn write(
    &mut self,
    buf: &[u8],
  ) -> std::io::Result<usize> {
    if self.tx_writer.send(buf.to_vec()).is_err() {
      return Err(io::Error::other("Failed to write"));
    }
    Ok(buf.len())
  }

  fn flush(&mut self) -> std::io::Result<()> {
    todo!()
  }
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
    if let Err(_error) = self.tx_writer.send(buf.to_vec()) {
      return std::task::Poll::Ready(Err(io::Error::other("Failed to write bytes")));
    };
    std::task::Poll::Ready(Ok(len))
  }

  fn poll_flush(
    self: std::pin::Pin<&mut Self>,
    cx: &mut std::task::Context<'_>,
  ) -> std::task::Poll<Result<(), io::Error>> {
    todo!()
  }

  fn poll_shutdown(
    self: std::pin::Pin<&mut Self>,
    cx: &mut std::task::Context<'_>,
  ) -> std::task::Poll<Result<(), io::Error>> {
    todo!()
  }
}

pub struct Http1AsyncWriter {
  pub(super) tx_writer: UnboundedSender<Vec<u8>>,
}

impl Write for Http1AsyncWriter {
  fn write(
    &mut self,
    buf: &[u8],
  ) -> std::io::Result<usize> {
    if self.tx_writer.send(buf.to_vec()).is_err() {
      return Err(io::Error::other("Failed to write"));
    }
    Ok(buf.len())
  }

  fn flush(&mut self) -> std::io::Result<()> {
    todo!()
  }
}

impl AsyncWrite for Http1AsyncWriter {
  fn poll_write(
    self: std::pin::Pin<&mut Self>,
    cx: &mut std::task::Context<'_>,
    buf: &[u8],
  ) -> std::task::Poll<Result<usize, io::Error>> {
    let len = buf.len();
    if let Err(_error) = self.tx_writer.send(buf.to_vec()) {
      return std::task::Poll::Ready(Err(io::Error::other("Failed to write bytes")));
    };
    std::task::Poll::Ready(Ok(len))
  }

  fn poll_flush(
    self: std::pin::Pin<&mut Self>,
    cx: &mut std::task::Context<'_>,
  ) -> std::task::Poll<Result<(), io::Error>> {
    todo!()
  }

  fn poll_shutdown(
    self: std::pin::Pin<&mut Self>,
    cx: &mut std::task::Context<'_>,
  ) -> std::task::Poll<Result<(), io::Error>> {
    todo!()
  }
}
