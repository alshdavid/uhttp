use std::io;
use std::pin::Pin;
use std::task::Context;
use std::task::Poll;

use http::HeaderMap;
use http::Method;
use http::Uri;
use http::Version;
use hyper::body::Body;
use hyper::body::Buf;
use tokio::io::AsyncRead;

pub struct Request {
  pub(crate) inner: Box<dyn AsyncRead + Unpin + Send + Sync>,
  pub(crate) headers: HeaderMap,
  pub(crate) method: Method,
  pub(crate) version: Version,
  pub(crate) uri: Uri,
}

impl Request {
  pub fn new(incoming: hyper::Request<hyper::body::Incoming>) -> Self {
    let (parts, body) = incoming.into_parts();
    Self {
      inner: Box::new(BodyReader::new(body)),
      headers: parts.headers,
      method: parts.method,
      version: parts.version,
      uri: parts.uri,
    }
  }

  pub fn body(&mut self) -> &mut Box<dyn AsyncRead + Unpin + Send + Sync> {
    &mut self.inner
  }

  pub fn headers(&self) -> &HeaderMap {
    &self.headers
  }

  pub fn method(&self) -> &Method {
    &self.method
  }

  pub fn version(&self) -> &Version {
    &self.version
  }

  pub fn uri(&self) -> &Uri {
    &self.uri
  }
}

struct BodyReader {
  body: hyper::body::Incoming,
  current_chunk: Option<hyper::body::Bytes>,
}

impl BodyReader {
  fn new(body: hyper::body::Incoming) -> Self {
    Self {
      body,
      current_chunk: None,
    }
  }
}

impl AsyncRead for BodyReader {
  fn poll_read(
    mut self: Pin<&mut Self>,
    cx: &mut Context<'_>,
    buf: &mut tokio::io::ReadBuf<'_>,
  ) -> Poll<io::Result<()>> {
    loop {
      // If we have a current chunk, read from it
      if let Some(chunk) = &mut self.current_chunk {
        if chunk.has_remaining() {
          let to_read = std::cmp::min(chunk.remaining(), buf.remaining());
          buf.put_slice(&chunk.chunk()[..to_read]);
          chunk.advance(to_read);
          return Poll::Ready(Ok(()));
        } else {
          // Current chunk is exhausted
          self.current_chunk = None;
        }
      }

      // Try to get the next frame
      let body_pin = Pin::new(&mut self.body);
      match body_pin.poll_frame(cx) {
        Poll::Ready(Some(Ok(frame))) => {
          if let Ok(data) = frame.into_data() {
            self.current_chunk = Some(data);
            continue;
          }
          // If it's not data (e.g., trailers), continue to next frame
          continue;
        }
        Poll::Ready(Some(Err(e))) => {
          return Poll::Ready(Err(io::Error::other(e)));
        }
        Poll::Ready(None) => {
          // End of stream
          return Poll::Ready(Ok(()));
        }
        Poll::Pending => {
          return Poll::Pending;
        }
      }
    }
  }
}
