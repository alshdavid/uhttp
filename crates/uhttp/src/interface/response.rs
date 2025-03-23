use std::io::Write;

use http::HeaderMap;
use tokio::io::AsyncWrite;

pub trait Response: Send + Sync + Unpin + AsyncWrite {
  fn headers_ref(&self) -> &HeaderMap;
  fn headers_mut(&mut self) -> &mut HeaderMap;
  fn set_header(&mut self, key: &str, value: &str);
  fn split_writer(&mut self) -> Box<dyn AsyncWrite + Send + Sync + Unpin>;
}
