# µHTTP 🦀🚀🌎

A fast, tiny library for writing HTTP servers in Rust designed for humans:

- **Simple:** Inspired by Go's standard library HTTP server.

- **Fast:** High performance, multi-threaded implementation built on top of Tokio that competes with the fastest Rust HTTP servers.

- **Flexible**: Simple interface that enables many use cases. It can be used directly or to act as a base for frameworks to build on top of.

## Installation

Available on [crates.io](https://crates.io/crates/uhttp), install with:

```shell
cargo add uhttp
```

## Benchmarks

Running on my desktop PC with a 7950x

```
oha -n 500000 -c 500 --latency-correction http://localhost:8080
```

```mermaid
---
config:
  xyChart:
    showTick: true
---
xychart-beta
    title "Requests Per Second"
    x-axis ["hyper", "uhttp", "Go HTTP", "Node.js (uws library)", "Node.js (std)"]
    y-axis "Requests Per Second" 0 --> 500000
    bar [497432, 493671, 412250, 140929, 79966]
```

# Usage

## Get Request

```rust
use std::io;

use uhttp::http1::Server;
use uhttp::HttpWriter;
use uhttp::Request;
use uhttp::Response;

#[tokio::main]
async fn main() -> io::Result<()> {
  Server::new(handler).listen("0.0.0.0:8080").await
}

async fn handler(mut _req: Request, mut res: Response) -> io::Result<()> {
  res.write_all(b"Hello World!").await?;
  res.write_header(200).await
}
```

## Request Body

```rust
use std::io;

use uhttp::http1::Server;
use uhttp::Request;
use uhttp::Response;

#[tokio::main]
async fn main() -> io::Result<()> {
  Server::new(handler).listen("0.0.0.0:8080").await
}

async fn handler(mut req: Request, mut res: Response) -> io::Result<()> {
  let body_text = uhttp::utils::body::utf8(&mut req).await?;
  println!("{}", body_text);

  res.write_header(201).await
}
```

## Routing

The URL is passed into the handler as a `String` and can be used to match request paths to routes. You can use simple if statements or a third party URL matching library to handle routing.

_TODO: Adding a basic router_

```rust
use std::io;

use uhttp::http1::Server;
use uhttp::HttpWriter;
use uhttp::Request;
use uhttp::Response;

#[tokio::main]
async fn main() -> io::Result<()> {
  Server::new(handler).listen("0.0.0.0:8080").await
}

async fn handler(mut req: Request, mut res: Response) -> io::Result<()> {
  if req.method == "GET" && req.url == "/" {
    return res.write_all(b"Hello World!").await
  }

  if req.method == "POST" && req.url == "/api/echo" {
    let bytes = uhttp::utils::body::bytes(&mut req).await?;
    return res.write_all(&bytes).await
  }

  res.write_header(404).await
}
```

## Serving a File

```rust
use std::io;
use std::path::PathBuf;

use tokio::fs;

use uhttp::http1::Server;
use uhttp::HttpWriter;
use uhttp::Request;
use uhttp::Response;

const CARGO_HOME: &str = env!("CARGO_MANIFEST_DIR");

#[tokio::main]
async fn main() -> io::Result<()> {
  Server::new(handler).listen("0.0.0.0:8080").await
}

async fn handler(_req: Request, mut res: Response) -> io::Result<()> {
  let index_file = PathBuf::from(CARGO_HOME).join("index.html");
  let bytes = fs::read(&index_file).await?;
  res.write_all(&bytes).await?;
  res.write_header(200).await
}
```

## Constants

Provided are some constants to make responses more consistent

```rust
use std::io;
use std::io::Write;

use uhttp::http1::Server;
use uhttp::Request;
use uhttp::Response;
use uhttp::c;

fn main() -> io::Result<()> {
  Server::new(handler).listen("0.0.0.0:8080")
}

fn handler(req: Request, mut res: Response) -> io::Result<()> {
  res.headers().set(c::headers::CONTENT_TYPE, c::content_type::TEXT_PLAIN);
  res.write_all(b"Hello World!")
  res.write_header(c::status::OK)
}
```

# Performance

## Setting Headers Explicitly

Setting the `Content-Type`, `Content-Length` or `Transfer-Encoding` explicitly will improve performance as the server does not need to detect them automatically.

### TODO

- Provide compressor utils for `Content-Encoding`: `gzip` and `br`
- `Transfer-Encoding: chunked`
- [Server Sent Events](https://developer.mozilla.org/en-US/docs/Web/API/Server-sent_events/Using_server-sent_events) (use this instead of WebSocket)
- HTTP/2
- More performance improvements

### Out of Scope

Though feel free to raise a PR to add support

- WebSocket
