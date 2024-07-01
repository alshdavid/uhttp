use std::io::Read;

use super::Headers;

pub struct Request {
  pub method: String,
  pub url: String,
  pub proto: String,
  pub proto_major: u8,
  pub proto_minor: u8,
  pub headers: Headers,
  pub body: Box<dyn Read>,
  pub host: String,
}
