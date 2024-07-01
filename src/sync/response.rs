use std::io::Write;
use std::io::{self};
use std::net::{Shutdown, TcpStream};

use crate::Headers;

pub trait Response: Write + Send + Sync {
  fn headers(&mut self) -> &mut Headers;
  fn write_header(
    &mut self,
    status_code: usize,
  ) -> io::Result<()>;
  fn end(&mut self) -> io::Result<()>;
}

pub struct HttpResponse {
  pub(super) headers: Headers,
  pub(super) stream: TcpStream,
}

impl Response for HttpResponse {
  fn headers(&mut self) -> &mut Headers {
    &mut self.headers
  }

  fn end(&mut self) -> io::Result<()> {
    self.stream.flush()?;
    self.stream.shutdown(Shutdown::Both)
  }

  fn write_header(
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

impl Write for HttpResponse {
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
