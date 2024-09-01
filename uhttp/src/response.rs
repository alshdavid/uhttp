// TODO rewrite the Go docs below and implement the approach

use std::io;

use tokio::sync::mpsc::UnboundedSender;

use crate::Headers;

#[async_trait::async_trait]
pub trait HttpWriter: Send + Sync {
  async fn write(&mut self, buf: &[u8]) -> io::Result<usize>;
  async fn write_all(&mut self, buf: &[u8]) -> io::Result<()>;
  async fn flush(&mut self) -> io::Result<()>;
}
pub struct Response {
  pub (crate) head: Option<Vec<String>>,
  pub (crate) body_buf: Vec<u8>,
  pub (crate) headers: Headers,
  pub (crate) writer: UnboundedSender<Vec<u8>>,
}

impl Response {
  pub fn write_header(
    &mut self,
    status_code: usize,
  ) -> io::Result<()> {
    let mut head = vec![];
    head.push(format!("HTTP/1.1 {status_code} OK"));

    for (key, values) in self.headers.iter() {
      head.push(format!("{}: {}", key, values.join(", ")));
    }

    self.head = Some(head);
    Ok(())
  }
  
  pub fn header(
    &mut self,
    key: &str,
    value: &str,
  ) {
    self.headers.set(key, value)
  }
}

#[async_trait::async_trait]
impl HttpWriter for Response {
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
  async fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
    self.body_buf.extend(buf);
    Ok(buf.len())
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
  async fn write_all(&mut self, buf: &[u8]) -> io::Result<()> {
    self.body_buf.extend(buf);
    Ok(())
  }
  
  async fn flush(&mut self) -> io::Result<()> {
    todo!()
  }
}

impl Drop for Response {
    fn drop(&mut self) {
      let writer = self.writer.clone();
      let mut head = std::mem::take(&mut self.head);
      let mut body_buf = std::mem::take(&mut self.body_buf);
      
      tokio::task::spawn(async move {
        let message = head.take().unwrap();
        let mut message = message.join("\r\n");
        message.push_str("\r\n");
        message.push_str(&format!("Content-Length: {}\r\n", body_buf.len()));
        message.push_str("\r\n");
        let mut message = message.as_bytes().to_vec();
        message.extend(body_buf.drain(0..));
        writer.send(message).unwrap();
      });
    }
}