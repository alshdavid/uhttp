mod tokio_minihttp;

use futures;
use tokio_proto;
use tokio_service;

use std::io;

use futures::future;
use tokio_minihttp::{Request, Response, Http};
use tokio_proto::TcpServer;
use tokio_service::Service;

/// `HelloWorld` is the *service* that we're going to be implementing to service
/// the HTTP requests we receive.
///
/// The tokio-minihttp crate, and much of Tokio itself, are centered around the
/// concept of a service for interoperability between crates. Our service here
/// carries no data with it.
///
/// Note that a new instance of `HelloWorld` is created for each TCP connection
/// we service, created below in the closure passed to `serve`.
struct HelloWorld;

impl Service for HelloWorld {
    type Request = Request;
    type Response = Response;
    type Error = io::Error;
    type Future = future::Ok<Response, io::Error>;

    fn call(&self, _request: Request) -> Self::Future {
        let mut resp = Response::new();
        resp.body("Hello, world!");
        future::ok(resp)
    }
}

fn main() {
    let addr = "0.0.0.0:8080".parse().unwrap();
    TcpServer::new(Http, addr)
        .serve(|| Ok(HelloWorld));
}