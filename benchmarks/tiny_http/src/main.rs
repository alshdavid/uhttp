extern crate tiny_http;

use std::sync::Arc;
use std::thread;

fn main() {
    let server = Arc::new(tiny_http::Server::http("0.0.0.0:8080").unwrap());
    println!("Now listening on port 8080");

    for rq in server.incoming_requests() {
      let response = tiny_http::Response::from_string("hello world".to_string());
      let _ = rq.respond(response);
    }
}