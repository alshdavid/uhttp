// TODO rewrite the Go docs below and implement the approach

use std::io;
use std::io::Write;

use crate::Headers;

pub struct Response {
  headers: Headers,
  response_writer: Box<dyn Write + Send>,
}

impl Response {
  pub fn new<T: Write + Send + 'static>(response_writer: T) -> Self {
    Self {
      headers: Default::default(),
      response_writer: Box::new(response_writer),
    }
  }
  pub fn headers(&mut self) -> &mut Headers {
    &mut self.headers
  }

  pub fn header<K: AsRef<str>, V: AsRef<str>>(
    &mut self,
    key: K,
    value: V,
  ) {
    self.headers.set(key, value)
  }

  /// WriteHeader sends an HTTP response header with the provided status code.

  /// If WriteHeader is not called explicitly, the first call to Write will
  /// trigger an implicit WriteHeader(http.StatusOK). Thus explicit calls to
  /// WriteHeader are mainly used to send error codes or 1xx informational
  /// responses.

  /// The provided code must be a valid HTTP 1xx-5xx status code. Any number
  /// of 1xx headers may be written, followed by at most one 2xx-5xx header.
  /// 1xx headers are sent immediately, but 2xx-5xx headers may be buffered.
  ///
  /// Use the Flusher interface to send buffered data. The header map is
  /// cleared when 2xx-5xx headers are sent, but not with 1xx headers.

  /// The server will automatically send a 100 (Continue) header on the first
  /// read from the request body if the request has an "Expect: 100-continue"
  /// header.
  pub fn write_header(
    &mut self,
    status_code: usize,
  ) -> io::Result<()> {
    let response_code = format!("{}", status_code);
    let response_message = "OK";

    let mut message = Vec::<u8>::new();

    message.extend(b"HTTP/1.1 ");
    message.extend(response_code.as_bytes());
    message.extend(b" ");

    message.extend(response_message.as_bytes());
    message.extend(b"\r\n");

    for (key, values) in self.headers.iter() {
      message.extend(key.as_bytes());
      message.extend(b": ");

      for value in values.iter() {
        message.extend(value.as_bytes());
        message.extend(b"\r\n");
      }
    }

    message.extend(b"\r\n");
    self.response_writer.write_all(&message)?;

    Ok(())
  }
}

impl Write for Response {
  /// Write writes the data to the connection as part of an HTTP reply.

  /// If [ResponseWriter.WriteHeader] has not yet been called, Write calls
  /// WriteHeader(http.StatusOK) before writing the data. If the Header does
  /// not contain a Content-Type line, Write adds a Content-Type set to the
  /// result of passing the initial 512 bytes of written data to [DetectContentType].
  /// Additionally, if the total size of all written data is under a few KB and
  /// there are no Flush calls, the Content-Length header is added automatically.

  /// Depending on the HTTP protocol version and the client, calling Write or
  /// WriteHeader may prevent future reads on the Request.Body.
  ///
  /// For HTTP/1.x requests, handlers should read any needed request body data
  /// before writing the response. Once the headers have been flushed
  /// (due to either an explicit Flusher.Flush call or writing enough data to
  /// trigger a flush), the request body may be unavailable. For HTTP/2 requests,
  /// the Go HTTP server permits handlers to continue to read the request body
  /// while concurrently writing the response. However, such behavior may not be
  /// supported by all HTTP/2 clients. Handlers should read before writing if
  /// possible to maximize compatibility.
  fn write(
    &mut self,
    buf: &[u8],
  ) -> io::Result<usize> {
    self.response_writer.write(buf)
  }

  /// Write writes the data to the connection as part of an HTTP reply.

  /// If [ResponseWriter.WriteHeader] has not yet been called, Write calls
  /// WriteHeader(http.StatusOK) before writing the data. If the Header does
  /// not contain a Content-Type line, Write adds a Content-Type set to the
  /// result of passing the initial 512 bytes of written data to [DetectContentType].
  /// Additionally, if the total size of all written data is under a few KB and
  /// there are no Flush calls, the Content-Length header is added automatically.

  /// Depending on the HTTP protocol version and the client, calling Write or
  /// WriteHeader may prevent future reads on the Request.Body.
  ///
  /// For HTTP/1.x requests, handlers should read any needed request body data
  /// before writing the response. Once the headers have been flushed
  /// (due to either an explicit Flusher.Flush call or writing enough data to
  /// trigger a flush), the request body may be unavailable. For HTTP/2 requests,
  /// the Go HTTP server permits handlers to continue to read the request body
  /// while concurrently writing the response. However, such behavior may not be
  /// supported by all HTTP/2 clients. Handlers should read before writing if
  /// possible to maximize compatibility.
  fn write_all(
    &mut self,
    buf: &[u8],
  ) -> io::Result<()> {
    self.response_writer.write_all(buf)
  }

  fn flush(&mut self) -> io::Result<()> {
    self.response_writer.flush()
  }
}

impl Drop for Response {
  fn drop(&mut self) {
    self.response_writer.flush().ok();
  }
}
