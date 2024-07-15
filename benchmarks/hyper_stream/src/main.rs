use std::net::{SocketAddr, TcpListener};

use hyper::server::conn::http1;

fn main() {
  // let addr = SocketAddr::from(([127, 0, 0, 1], 8080));

  // let listener = TcpListener::bind(addr).unwrap();

  // while let Ok((stream, _)) = listener.accept() {
  //   http1::Builder::new()
  //     .serve_connection(io, service_fn(root))
  //     .await
  //     .unwrap();
  // }
}
