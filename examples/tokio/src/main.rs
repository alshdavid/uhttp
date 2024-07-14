use std::convert::Infallible;
use std::net::SocketAddr;

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
use tokio::net::TcpListener;

fn main() {
  tokio::runtime::Builder::new_multi_thread()
    .enable_all()
    .worker_threads(num_cpus::get_physical())
    .build()
    .unwrap()
    .block_on(async {
      let addr = SocketAddr::from(([127, 0, 0, 1], 8080));
      let listener = TcpListener::bind(addr).await.unwrap();

      loop {
        let (stream, _) = listener.accept().await.unwrap();
        let io = TokioIo::new(stream);

        tokio::task::spawn(async move {
          http1::Builder::new()
            .serve_connection(io, service_fn(root))
            .await
            .unwrap();
        });
      }
    });
}

async fn root(_request: Request<Incoming>) -> Result<Response<BoxBody<Bytes, std::io::Error>>, Infallible> {
  let (mut writer, reader) = tokio::io::duplex(1024);
  let reader_stream = tokio_util::io::ReaderStream::new(reader);
  let stream_body = StreamBody::new(reader_stream.map_ok(hyper::body::Frame::data));
  let boxed_body = stream_body.boxed();

  let response = hyper::Response::builder()
    .header("Content-Type", "text/html")
    .status(hyper::StatusCode::OK)
    .body(boxed_body)
    .unwrap();

  // tokio::task::spawn(async move {
    writer.write(b"hello world").await.unwrap();
  // });

  Ok(response)
}
