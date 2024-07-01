use std::io::Write;
use std::io;
use std::net::TcpStream;
// use tokio::io::Write;

use tokio::io::{AsyncWrite, AsyncWriteExt};

use crate::Headers;

pub trait Response {
  fn headers(&mut self) -> &mut Headers;
  // fn message<S: AsRef<str>>(&mut self, message: S) -> &mut Headers;
  fn write_header(
    &mut self,
    status_code: usize,
  ) -> io::Result<()>;
}

pub struct HttpResponse {
  pub(super) headers: Headers,
  pub(super) stream: Box<dyn AsyncWrite>,
}

impl Response for HttpResponse {
  fn headers(&mut self) -> &mut Headers {
    &mut self.headers
  }

  fn write_header(
    &mut self,
    status_code: usize,
  ) -> io::Result<()> {
    let response_code = format!("{}", status_code);
    let response_message = "OK";

    // // Write Reply
    // self.stream.write_all(b"HTTP/1.1 ")?;
    // // Status Code
    // self.stream.write_all(response_code.as_bytes())?;
    // self.stream.write_all(b" ")?;

    // // Message
    // self.stream.write_all(response_message.as_bytes())?;
    // self.stream.write_all(b"\r\n")?;

    // for (key, values) in self.headers.iter() {
    //   self.stream.write_all(key.as_bytes())?;
    //   self.stream.write_all(b": ")?;

    //   self.stream.write_all(values.as_bytes())?;

    //   // for (i, value) in values.iter().enumerate() {
    //   //   self.stream.write_all(value.as_bytes())?;
    //   //   if i != values.len() - 1 {
    //   //     self.stream.write_all(b", ")?;
    //   //   }
    //   // }

    //   self.stream.write_all(b"\r\n")?;
    // }

    // self.stream.write_all(b"\r\n")?;

    Ok(())
  }

  // fn message<S: AsRef<str>>(&mut self, _message: S) -> &mut Headers {
  //     todo!()
  //   }
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
