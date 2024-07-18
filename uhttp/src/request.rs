use std::io::Read;

use crate::Headers;

pub struct Request {
  pub method: String,
  pub url: String,
  pub proto: String,
  pub headers: Headers,
  pub body: Box<dyn Read + Send>,
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
