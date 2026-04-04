use std::convert::Infallible;
use std::io;
use std::pin::Pin;
use std::sync::Arc;
use std::task::Context;
use std::task::Poll;

use futures::TryStreamExt;
use http::HeaderMap;
use http::HeaderName;
use http::HeaderValue;
use tokio::io::AsyncWrite;
use tokio::io::AsyncWriteExt;
use tokio::sync::Mutex;
use tokio::sync::RwLock;

use super::Headers;

pub(super) enum ResponseState {
  Builder(
    (
      hyper::http::response::Builder,
      tokio::sync::oneshot::Sender<
        http::Response<http_body_util::combinators::BoxBody<hyper::body::Bytes, Infallible>>,
      >,
    ),
  ),
  Stream(tokio::io::DuplexStream),
  Done,
  Pending,
}

#[derive(Clone)]
pub struct Response {
  state: Arc<RwLock<ResponseState>>,
  buffer: Arc<Mutex<Vec<u8>>>,
}

impl Response {
  pub fn new(
    tx_res: tokio::sync::oneshot::Sender<
      http::Response<http_body_util::combinators::BoxBody<hyper::body::Bytes, Infallible>>,
    >,
    builder: hyper::http::response::Builder,
  ) -> Self {
    Self {
      state: Arc::new(RwLock::new(ResponseState::Builder((builder, tx_res)))),
      buffer: Default::default(),
    }
  }

  pub fn header(&self) -> Headers {
    Headers::new(Arc::clone(&self.state))
  }

  pub async fn get_header(
    &self,
    name: &str,
  ) -> Option<String> {
    let guard = self.state.read().await;
    match &*guard {
      ResponseState::Builder((builder, _)) => {
        let Some(headers) = builder.headers_ref() else {
          return None;
        };
        headers
          .get(name)
          .and_then(|v| v.to_str().ok())
          .map(|s| s.to_string())
      }
      _ => None,
    }
  }

  pub async fn get_all_headers(
    &self,
    name: &str,
  ) -> Vec<String> {
    let guard = self.state.read().await;
    match &*guard {
      ResponseState::Builder((builder, _)) => {
        let Some(headers) = builder.headers_ref() else {
          return Vec::new();
        };
        headers
          .get_all(name)
          .into_iter()
          .filter_map(|v| v.to_str().ok())
          .map(|s| s.to_string())
          .collect()
      }
      _ => Vec::new(),
    }
  }

  pub async fn insert_header(
    &self,
    key: &str,
    value: &str,
  ) -> anyhow::Result<Option<String>> {
    let mut guard = self.state.write().await;
    match &mut *guard {
      ResponseState::Builder((builder, _)) => {
        let Some(headers) = builder.headers_mut() else {
          return Err(anyhow::anyhow!("No headers"));
        };
        let Ok(header) = HeaderValue::from_str(value) else {
          return Err(anyhow::anyhow!("Invalid header value"));
        };
        let Ok(key) = HeaderName::from_bytes(key.as_bytes()) else {
          return Err(anyhow::anyhow!("Invalid header key"));
        };
        if let Some(prev) = headers.insert(key, header) {
          if let Ok(prev) = prev.to_str() {
            return Ok(Some(prev.to_string()));
          }
          return Err(anyhow::anyhow!("Unable to parse existing header"));
        };
        return Ok(None);
      }
      ResponseState::Stream(_) => {
        return Err(anyhow::anyhow!("Headers already sent"));
      }
      ResponseState::Done => {
        return Err(anyhow::anyhow!("Request already sent"));
      }
      ResponseState::Pending => {
        return Err(anyhow::anyhow!("Request currently sending"));
      }
    };
  }

  pub async fn append_header(
    &self,
    key: &str,
    value: &str,
  ) -> anyhow::Result<bool> {
    let mut guard = self.state.write().await;
    match &mut *guard {
      ResponseState::Builder((builder, _)) => {
        let Some(headers) = builder.headers_mut() else {
          return Err(anyhow::anyhow!("No headers"));
        };
        let Ok(header) = HeaderValue::from_str(value) else {
          return Err(anyhow::anyhow!("Invalid header value"));
        };
        let Ok(key) = HeaderName::from_bytes(key.as_bytes()) else {
          return Err(anyhow::anyhow!("Invalid header key"));
        };
        return Ok(headers.append(key, header));
      }
      ResponseState::Stream(_) => {
        return Err(anyhow::anyhow!("Headers already sent"));
      }
      ResponseState::Done => {
        return Err(anyhow::anyhow!("Request already sent"));
      }
      ResponseState::Pending => {
        return Err(anyhow::anyhow!("Request currently sending"));
      }
    };
  }

  /// Send the headers and the body. Headers cannot be sent after this is called
  pub async fn write_head(
    &self,
    status: http::StatusCode,
  ) -> anyhow::Result<()> {
    let mut guard = self.state.write().await;

    let (mut builder, tx_res) = match std::mem::replace(&mut *guard, ResponseState::Pending) {
      ResponseState::Builder(builder) => builder,
      ResponseState::Stream(_response) => return Err(anyhow::anyhow!("Already wrote head")),
      ResponseState::Done => return Err(anyhow::anyhow!("Response has ended")),
      ResponseState::Pending => return Err(anyhow::anyhow!("Currently Writing")),
    };

    builder = builder.status(status);

    let (mut writer, reader) = tokio::io::duplex(512);

    let reader_stream = tokio_util::io::ReaderStream::new(reader)
      .map_ok(hyper::body::Frame::data)
      .map_err(|_item| panic!());

    let stream_body = http_body_util::StreamBody::new(reader_stream);
    let boxed_body: http_body_util::combinators::BoxBody<hyper::body::Bytes, Infallible> =
      http_body_util::combinators::BoxBody::<hyper::body::Bytes, Infallible>::new(stream_body);

    let res: http::Response<http_body_util::combinators::BoxBody<hyper::body::Bytes, Infallible>> =
      builder.body(boxed_body)?;

    if tx_res.send(res).is_err() {
      return Err(anyhow::anyhow!("Failed to send request"));
    };

    let mut buffer = self.buffer.lock().await;
    if !buffer.is_empty() {
      let b = std::mem::take(&mut (*buffer));
      writer.write_all(b.as_slice()).await?;
    }

    drop(std::mem::replace(
      &mut *guard,
      ResponseState::Stream(writer),
    ));

    Ok(())
  }

  /// End the http response, nothing can be sent after this is called
  pub async fn end(&self) -> anyhow::Result<()> {
    let mut guard = self.state.write().await;

    let inner = std::mem::replace(&mut *guard, ResponseState::Pending);
    match inner {
      ResponseState::Builder(_builder) => return Err(anyhow::anyhow!("Already wrote head")),
      ResponseState::Stream(_response) => return Err(anyhow::anyhow!("Already wrote head")),
      ResponseState::Done => return Err(anyhow::anyhow!("Response has ended")),
      ResponseState::Pending => return Err(anyhow::anyhow!("Currently Writing")),
    };
  }
}

impl Drop for Response {
  fn drop(&mut self) {
    let state = Arc::clone(&self.state);
    let buffer = Arc::clone(&self.buffer);

    tokio::task::spawn(async move {
      let mut guard = state.write().await;
      let inner = std::mem::replace(&mut *guard, ResponseState::Pending);

      match inner {
        ResponseState::Builder((builder, tx_res)) => {
          let mut buffer = buffer.lock().await;
          let bytes = std::mem::take(&mut (*buffer));
          let b = hyper::body::Bytes::from(bytes);
          let b2 = http_body_util::Full::new(b);
          let body = http_body_util::combinators::BoxBody::new(b2);
          let res = builder.status(200).body(body).unwrap();
          tx_res.send(res).unwrap();
        }
        ResponseState::Stream(_response) => {}
        ResponseState::Done => {}
        ResponseState::Pending => {}
      };
    });
  }
}

impl AsyncWrite for Response {
  // If "write_head" has not been called, buffer the body and send it
  // the first time write is called. Subsequent calls to write are streamed
  // to the client
  fn poll_write(
    self: Pin<&mut Self>,
    cx: &mut Context<'_>,
    buf: &[u8],
  ) -> Poll<Result<usize, io::Error>> {
    let this = self.get_mut();

    let mut state_guard = match this.state.try_write() {
      Ok(guard) => guard,
      Err(_) => {
        cx.waker().wake_by_ref();
        return Poll::Pending;
      }
    };

    match &mut *state_guard {
      ResponseState::Stream(writer) => Pin::new(writer).poll_write(cx, buf),
      ResponseState::Builder(_) => {
        let mut buffer_guard = match this.buffer.try_lock() {
          Ok(guard) => guard,
          Err(_) => {
            cx.waker().wake_by_ref();
            return Poll::Pending;
          }
        };
        buffer_guard.extend_from_slice(buf);
        Poll::Ready(Ok(buf.len()))
      }
      ResponseState::Done => Poll::Ready(Err(io::Error::new(
        io::ErrorKind::BrokenPipe,
        "Response has ended",
      ))),
      ResponseState::Pending => Poll::Ready(Err(io::Error::new(
        io::ErrorKind::WouldBlock,
        "Currently writing",
      ))),
    }
  }

  fn poll_flush(
    self: Pin<&mut Self>,
    cx: &mut Context<'_>,
  ) -> Poll<Result<(), io::Error>> {
    let this = self.get_mut();

    let mut state_guard = match this.state.try_write() {
      Ok(guard) => guard,
      Err(_) => {
        cx.waker().wake_by_ref();
        return Poll::Pending;
      }
    };

    match &mut *state_guard {
      ResponseState::Stream(writer) => Pin::new(writer).poll_flush(cx),
      ResponseState::Builder(_) => Poll::Ready(Ok(())),
      ResponseState::Done => Poll::Ready(Err(io::Error::new(
        io::ErrorKind::BrokenPipe,
        "Response has ended",
      ))),
      ResponseState::Pending => Poll::Ready(Err(io::Error::new(
        io::ErrorKind::WouldBlock,
        "Currently writing",
      ))),
    }
  }

  fn poll_shutdown(
    self: Pin<&mut Self>,
    cx: &mut Context<'_>,
  ) -> Poll<Result<(), io::Error>> {
    let this = self.get_mut();

    // Try to lock the state non-blockingly
    let mut state_guard = match this.state.try_write() {
      Ok(guard) => guard,
      Err(_) => {
        // Lock is held, wake and return pending
        cx.waker().wake_by_ref();
        return Poll::Pending;
      }
    };

    match &mut *state_guard {
      ResponseState::Stream(writer) => {
        // Shutdown the underlying stream
        match Pin::new(writer).poll_shutdown(cx) {
          Poll::Ready(Ok(())) => {
            // Transition to Done state
            *state_guard = ResponseState::Done;
            Poll::Ready(Ok(()))
          }
          Poll::Ready(Err(e)) => Poll::Ready(Err(e)),
          Poll::Pending => Poll::Pending,
        }
      }
      ResponseState::Builder(_) => Poll::Ready(Err(io::Error::other(
        "Cannot shutdown before write_head is called",
      ))),
      ResponseState::Done => {
        // Already shut down
        Poll::Ready(Ok(()))
      }
      ResponseState::Pending => Poll::Ready(Err(io::Error::new(
        io::ErrorKind::WouldBlock,
        "Currently writing",
      ))),
    }
  }
}
