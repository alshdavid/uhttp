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

## Usage

TODO but the desired API is this:

```rust
use tokio::io::AsyncWriteExt;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
  uhttp::http1::create_server(async |req, res| {
    res.write(b"hello world\n").await
  })
    .listen("0.0.0.0:8080")
    .await
}
```

```rust
use tokio::io::AsyncWriteExt;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
  let mut app = uhttp::mux::Router();

  app.get("/foo", async |req, res| {
    res.write(b"bar\n").await
  })

  app.post("/bar", async |req, res| {
    res.write(b"foo\n").await
  })

  uhttp::http1::create_server(app.handler())
    .listen("0.0.0.0:8080")
    .await
}
```