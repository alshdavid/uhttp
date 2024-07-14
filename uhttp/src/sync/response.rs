use std::io::Write;
use std::io::{self};
use std::net::Shutdown;
use std::net::TcpStream;

use crate::Headers;

pub struct Response {
  pub(super) headers: Headers,
  pub(super) stream: TcpStream,
  pub (super) alive: bool
}

impl Response {
  pub fn new(
    headers: Headers,
    stream: TcpStream,
  ) -> Self {
    Self { 
      headers,
      stream,
      alive: true,
    }
  }
  pub fn headers(&mut self) -> &mut Headers {
    &mut self.headers
  }

  pub fn end(&mut self) -> io::Result<()> {
    if !self.alive {
      return Ok(());
    }
    self.stream.flush()?;
    self.stream.shutdown(Shutdown::Both)
  }

  pub fn write_header(
    &mut self,
    status_code: usize,
  ) -> io::Result<()> {
    let response_code = format!("{}", status_code);
    let response_message = "OK";

    // Write Reply
    self.stream.write(b"HTTP/1.1 ")?;
    // Status Code
    self.stream.write(response_code.as_bytes())?;
    self.stream.write(b" ")?;

    // Message
    self.stream.write(response_message.as_bytes())?;
    self.stream.write(b"\r\n")?;

    for (key, values) in self.headers.iter() {
      self.stream.write(key.as_bytes())?;
      self.stream.write(b": ")?;

      self.stream.write(values.as_bytes())?;
      self.stream.write(b"\r\n")?;
    }

    self.stream.write(b"\r\n")?;
    self.stream.flush()?;

    Ok(())
  }
}

impl Write for Response {
  fn write(
    &mut self,
    buf: &[u8],
  ) -> io::Result<usize> {
    self.stream.write(buf)
  }

  fn flush(&mut self) -> io::Result<()> {
    self.stream.flush()
  }
}

impl Drop for Response {
    fn drop(&mut self) {
      self.end().unwrap();
    }
}