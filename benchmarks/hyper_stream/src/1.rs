use std::collections::HashMap;
use std::convert::Infallible;
use std::net::SocketAddr;
use std::pin::Pin;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::time::Duration;

use futures::Future;
use futures::TryStreamExt;
use http_body_util::combinators::BoxBody;
use http_body_util::BodyExt;
use http_body_util::StreamBody;
use hyper::body::Bytes;
use hyper::body::Incoming;
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::Request;
use hyper::Response;
use hyper_util::rt::TokioIo;
use tokio::io::AsyncWriteExt;
use tokio::io::DuplexStream;
use tokio::net::TcpListener;
use tokio::sync::Mutex;

static COUNTER: AtomicUsize = AtomicUsize::new(0);

fn main() {
  tokio::runtime::Builder::new_current_thread()
    .enable_all()
    // .worker_threads(num_cpus::get_physical())
    .build()
    .unwrap()
    .block_on(async {
      let addr = SocketAddr::from(([127, 0, 0, 1], 8080));
      let listener = TcpListener::bind(addr).await.unwrap();

      let writers = Arc::new(Mutex::new(HashMap::<usize, DuplexStream>::new()));

      // let (tx, rx) = channel(1000);

      // tokio::task::spawn(async move {
      //   while let Some(_) = rx.recv().await {

      //   }
      // });

      loop {
        let writers = writers.clone();
        let (stream, _) = listener.accept().await.unwrap();
        let io = TokioIo::new(stream);

        // tokio::task::spawn(async move {
          http1::Builder::new()
            .serve_connection(io, service_fn(root(writers)))
            .await
            .unwrap();
        // });
      }
    });
}

fn root(
  writers: Arc<Mutex<HashMap<usize, DuplexStream>>>
) -> Box<
  dyn Fn(
      Request<Incoming>,
    ) -> Pin<
      Box<
        dyn Future<Output = Result<Response<BoxBody<Bytes, std::io::Error>>, Infallible>>
          + Send
          + Sync,
      >,
    > + Send
    + Sync,
> {
  return Box::new(move |_r| {
    let writers = writers.clone();
    Box::pin(async move {
      let (mut writer, reader) = tokio::io::duplex(1024);
      // {
      //   let writers = writers.lock().await;
      // }

      tokio::task::spawn(Box::pin(async move {
        writer.write(b"hello world").await.unwrap();
      }));

      //   // for i in 0..10 {
      //   //   writer.write(format!("{}", i).as_bytes()).await.unwrap();
      //   //   tokio::time::sleep(Duration::from_millis(500)).await;

      //   // }
      // }));

      let reader_stream = tokio_util::io::ReaderStream::new(reader);
      let stream_body = StreamBody::new(reader_stream.map_ok(hyper::body::Frame::data));
      let boxed_body = stream_body.boxed();

      let response = hyper::Response::builder()
        .header("Content-Type", "text/html")
        .status(hyper::StatusCode::OK)
        .body(boxed_body)
        .unwrap();

      Ok(response)
    })
  });
}
