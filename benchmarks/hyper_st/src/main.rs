use std::convert::Infallible;
use std::net::SocketAddr;

use http_body_util::Full;
use hyper::body::Bytes;
use hyper::body::Incoming;
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::Request;
use hyper::Response;
use hyper_util::rt::TokioIo;
use tokio::net::TcpListener;

fn main() {
  tokio::runtime::Builder::new_current_thread()
    .enable_all()
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

async fn root(_request: Request<Incoming>) -> Result<Response<Full<Bytes>>, Infallible> {
  Ok(Response::new(Full::new(Bytes::from("Hello, World!"))))
}
