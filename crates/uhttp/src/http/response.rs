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

    let to_write = {
      let mut state = inner.lock();

      let Some(ResponseState::Builder {
        mut builder,
        tx_res,
        mut buffer,
      }) = state.take()
      else {
        return crate::Error::generic_err("Tried to write head but head was already written");
      };

      builder = builder.status(status);

      let (writer, reader) = tokio::io::duplex(512);

      let reader_stream = tokio_util::io::ReaderStream::new(reader)
        .map_ok(hyper::body::Frame::data)
        .map_err(|_item| panic!());

      let stream_body = http_body_util::StreamBody::new(reader_stream);
      let boxed_body: http_body_util::combinators::BoxBody<hyper::body::Bytes, Infallible> =
        http_body_util::combinators::BoxBody::<hyper::body::Bytes, Infallible>::new(stream_body);

      let res: http::Response<
        http_body_util::combinators::BoxBody<hyper::body::Bytes, Infallible>,
      > = builder.body(boxed_body)?;

      if tx_res.send(res).is_err() {
        return Err(crate::Error::generic("Failed to send request"));
      };

      state.replace(ResponseState::Stream(writer));

      if !buffer.is_empty() {
        Some(std::mem::take(&mut buffer))
      } else {
        None
      }
    };

    if let Some(b) = to_write {
      self.write_all(b.as_slice()).await?;
    }
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

    let Some(ResponseState::Builder {
      builder,
      tx_res,
      mut buffer,
    }) = state.take()
    else {
      return;
    };

    println!("Dropping backup");

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
      ResponseState::Stream(writer) => Pin::new(writer).poll_flush(cx),
      ResponseState::Builder {
        builder: _,
        tx_res: _,
        buffer: _,
      } => Poll::Ready(Ok(())),
      ResponseState::Done => Poll::Ready(Err(io::Error::new(
        io::ErrorKind::BrokenPipe,
        "Response has ended",
      ))),
    }
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
      ResponseState::Done => Poll::Ready(Ok(())),
    }
  }
}
