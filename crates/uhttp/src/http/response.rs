use std::convert::Infallible;
use std::io;
use std::pin::Pin;
use std::sync::Arc;
use std::task::Context;
use std::task::Poll;

use futures::TryStreamExt;
use parking_lot::Mutex;
use tokio::io::AsyncWrite;
use tokio::io::AsyncWriteExt;

use super::Headers;

type HyperResponse =
  http::Response<http_body_util::combinators::BoxBody<hyper::body::Bytes, Infallible>>;

pub(crate) enum ResponseState {
  Builder {
    builder: hyper::http::response::Builder,
    tx_res: tokio::sync::oneshot::Sender<HyperResponse>,
    buffer: Vec<u8>,
  },
  Write {
    builder: hyper::http::response::Builder,
    tx_res: tokio::sync::oneshot::Sender<HyperResponse>,
    buffer: Vec<u8>,
  },
  Stream(tokio::io::DuplexStream),
  Done,
}

pub(crate) struct ResponseInner {
  state: ResponseState,
}

#[derive(Clone)]
pub struct Response {
  pub(crate) inner: Option<Arc<Mutex<Option<ResponseState>>>>,
}

impl Response {
  pub fn new(
    tx_res: tokio::sync::oneshot::Sender<
      http::Response<http_body_util::combinators::BoxBody<hyper::body::Bytes, Infallible>>,
    >,
    builder: hyper::http::response::Builder,
  ) -> Self {
    Self {
      inner: Some(Arc::new(Mutex::new(Some(ResponseState::Builder {
        builder: builder,
        tx_res: tx_res,
        buffer: Default::default(),
      })))),
    }
  }

  pub fn header(&self) -> Headers {
    // Headers::new(Arc::clone(&self.state))
    todo!()
  }

  /// Send the headers and the body. Headers cannot be sent after this is called
  pub async fn write_head(
    &mut self,
    status: http::StatusCode,
  ) -> crate::Result<()> {
    let Some(inner) = &self.inner else {
      return crate::Error::generic_err("Request Closed");
    };

    let mut state = inner.lock();

    let Some(ResponseState::Builder {
      builder,
      tx_res,
      buffer,
    }) = state.take()
    else {
      return crate::Error::generic_err("Tried to write head but head was already written");
    };

    state.replace(ResponseState::Write {
      builder: builder.status(status),
      tx_res,
      buffer,
    });

    Ok(())
  }

  /// End the http response, nothing can be sent after this is called
  pub async fn end(&self) -> crate::Result<()> {
    let Some(inner) = &self.inner else {
      return crate::Error::generic_err("Request Closed");
    };

    let mut state = inner.lock();
    drop(state.take());
    Ok(())
  }
}

impl Drop for Response {
  fn drop(&mut self) {
    let Some(inner) = self.inner.take() else {
      return;
    };

    let Some(inner) = Arc::into_inner(inner) else {
      return;
    };

    let mut state = inner.lock();

    let (builder, tx_res, mut buffer) = match state.take() {
      Some(ResponseState::Builder {
        builder,
        tx_res,
        buffer,
      }) => (builder, tx_res, buffer),
      Some(ResponseState::Write {
        builder,
        tx_res,
        buffer,
      }) => (builder, tx_res, buffer),
      _ => return,
    };

    let bytes = std::mem::take(&mut buffer);
    let b = hyper::body::Bytes::from(bytes);
    let b2 = http_body_util::Full::new(b);
    let body = http_body_util::combinators::BoxBody::new(b2);
    let res = builder.status(200).body(body).unwrap();
    drop(tx_res.send(res));
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

    let Some(inner) = &this.inner else {
      return Poll::Ready(Err(io::Error::new(
        io::ErrorKind::BrokenPipe,
        "Response has ended",
      )));
    };

    let mut state_guard = inner.lock();

    let Some(inner_guard) = &mut *state_guard else {
      return Poll::Ready(Err(io::Error::new(
        io::ErrorKind::BrokenPipe,
        "Response has ended",
      )));
    };

    match &mut *inner_guard {
      ResponseState::Stream(writer) => Pin::new(writer).poll_write(cx, buf),
      ResponseState::Builder {
        builder: _,
        tx_res: _,
        buffer,
      } => {
        buffer.extend_from_slice(buf);
        Poll::Ready(Ok(buf.len()))
      }
      ResponseState::Write {
        builder: _,
        tx_res: _,
        buffer,
      } => {
        buffer.extend_from_slice(buf);
        Poll::Ready(Ok(buf.len()))
      }
      ResponseState::Done => Poll::Ready(Err(io::Error::new(
        io::ErrorKind::BrokenPipe,
        "Response has ended",
      ))),
    }
  }

  fn poll_flush(
    self: Pin<&mut Self>,
    cx: &mut Context<'_>,
  ) -> Poll<Result<(), io::Error>> {
    let this = self.get_mut();

    let Some(inner) = &this.inner else {
      return Poll::Ready(Err(io::Error::new(
        io::ErrorKind::BrokenPipe,
        "Response has ended",
      )));
    };

    let mut state_guard = inner.lock();

    let Some(inner_guard) = &mut *state_guard else {
      return Poll::Ready(Err(io::Error::new(
        io::ErrorKind::BrokenPipe,
        "Response has ended",
      )));
    };

    match &mut *inner_guard {
      ResponseState::Stream(writer) => return Pin::new(writer).poll_flush(cx),
      ResponseState::Builder {
        builder: _,
        tx_res: _,
        buffer: _,
      } => return Poll::Ready(Ok(())),
      ResponseState::Write {
        builder: _,
        tx_res: _,
        buffer: _,
      } => {}
      ResponseState::Done => {
        return Poll::Ready(Err(io::Error::new(
          io::ErrorKind::BrokenPipe,
          "Response has ended",
        )));
      }
    }

    let Some(ResponseState::Write {
      builder,
      tx_res,
      buffer,
    }) = state_guard.take()
    else {
      unreachable!()
    };

    let (mut writer, reader) = tokio::io::duplex(512);

    let reader_stream = tokio_util::io::ReaderStream::new(reader)
      .map_ok(hyper::body::Frame::data)
      .map_err(|_item| panic!());

    if !buffer.is_empty() {
      let mut written = 0;
      while written < buffer.len() {
        match Pin::new(&mut writer).poll_write(cx, &buffer[written..]) {
          Poll::Ready(Ok(n)) => {
            if n == 0 {
              return Poll::Ready(Err(io::Error::new(
                io::ErrorKind::WriteZero,
                "failed to write buffered data to duplex stream",
              )));
            }
            written += n;
          }
          Poll::Ready(Err(e)) => return Poll::Ready(Err(e)),
          Poll::Pending => {
            // This should rarely happen on initialization unless the duplex capacity
            // is smaller than your buffer size. If it does happen, we must put the state
            // back so we can resume writing on the next poll.
            state_guard.replace(ResponseState::Write {
              builder,
              tx_res: tokio::sync::oneshot::channel().0, // The channel is already consumed
              buffer: buffer[written..].to_vec(),
            });
            return Poll::Pending;
          }
        }
      }
    }

    let stream_body = http_body_util::StreamBody::new(reader_stream);
    let boxed_body: http_body_util::combinators::BoxBody<hyper::body::Bytes, Infallible> =
      http_body_util::combinators::BoxBody::<hyper::body::Bytes, Infallible>::new(stream_body);

    let Ok(res) = builder.body(boxed_body) else {
      return Poll::Ready(Err(io::Error::new(
        io::ErrorKind::BrokenPipe,
        "Failed to send request",
      )));
    };

    if tx_res.send(res).is_err() {
      return Poll::Ready(Err(io::Error::new(
        io::ErrorKind::BrokenPipe,
        "Failed to send request",
      )));
    };

    let res = Pin::new(&mut writer).poll_flush(cx);
    state_guard.replace(ResponseState::Stream(writer));
    res
  }

  fn poll_shutdown(
    self: Pin<&mut Self>,
    cx: &mut Context<'_>,
  ) -> Poll<Result<(), io::Error>> {
    let this = self.get_mut();

    let Some(inner) = &this.inner else {
      return Poll::Ready(Err(io::Error::new(
        io::ErrorKind::BrokenPipe,
        "Response has ended",
      )));
    };

    let mut state_guard = inner.lock();

    let Some(inner_guard) = &mut *state_guard else {
      return Poll::Ready(Err(io::Error::new(
        io::ErrorKind::BrokenPipe,
        "Response has ended",
      )));
    };

    match &mut *inner_guard {
      ResponseState::Stream(writer) => match Pin::new(writer).poll_shutdown(cx) {
        Poll::Ready(Ok(())) => {
          state_guard.replace(ResponseState::Done);
          Poll::Ready(Ok(()))
        }
        Poll::Ready(Err(e)) => Poll::Ready(Err(e)),
        Poll::Pending => Poll::Pending,
      },
      ResponseState::Builder {
        builder: _,
        tx_res: _,
        buffer: _,
      } => Poll::Ready(Err(io::Error::other(
        "Cannot shutdown before write_head is called",
      ))),
      ResponseState::Write {
        builder: _,
        tx_res: _,
        buffer: _,
      } => Poll::Ready(Err(io::Error::other(
        "Cannot shutdown before write_head is called",
      ))),
      ResponseState::Done => Poll::Ready(Ok(())),
    }
  }
}
