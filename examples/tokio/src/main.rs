use std::sync::{atomic::{AtomicUsize, Ordering}, Arc};

use http::header::CONTENT_TYPE;
use server::{HttpServer, Request, Response};

#[macro_use]
extern crate log;

mod buffer;
mod date;

pub mod body;
pub mod server;


fn hello(_req: Request, res: &mut Response) {
  res.headers_mut().append(CONTENT_TYPE, "text/plain; charset=utf-8".parse().unwrap());
  res.send(b"Hello World!").unwrap();
}

fn main() {
  let server = HttpServer::new(hello).start("127.0.0.1:8080").unwrap();
  server.wait();
}