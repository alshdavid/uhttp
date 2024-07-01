use std::io::BufRead;
use super::Headers;

pub struct Request<'a> {
  pub method: String,
  pub url: String,
  pub proto: String,   // "HTTP/1.0"
  pub proto_major: u8, // 1
  pub proto_minor: u8, // 0
  pub headers: &'a mut Headers,
  pub body: Box<dyn BufRead>,
  pub host: String,
}
