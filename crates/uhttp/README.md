# µHTTP

`uhttp` is a lightweight and composible http server and router for building Rust HTTP services

- **Simple:** Inspired by Go's standard library HTTP server. Uses `Read` / `Write` traits for the `Request` / `Response`.

- **Fast:** High performance, multi-threaded implementation built on top of Hyper & Tokio that competes with the fastest Rust HTTP servers.

- **Flexible**: Simple interface that enables many use cases. It can be used directly or to act as a base for frameworks to build on top of.

## Installation

```shell
cargo add uhttp
cargo add uhttp -F query                      # Query string deserialization
cargo add uhttp -F json                       # JSON deserialization
cargo add uhttp -F router                     # Router for URLs
cargo add uhttp -F http2                      # Support for HTTP/2 with SSL
cargo add uhttp -F websocket                  # Support for Websockets
cargo add uhttp -F file_server                # File server that reads from the file system
cargo add uhttp -F file_server_include_dir    # File server that reads from embedded files
cargo add uhttp -F anyhow                     # Support for error handling with anyhow
```

## Usage

### Basic Response

```rust
use uhttp::*;

#[tokio::main]
async fn main() -> anyhow::Result<()> {

  uhttp::http1::create_server(|req, mut res| async move {
    res.header().add("Content-Type", "text/html").await?;
    res.write_all(b"<body>hello world</body>").await?;
    Ok(())
  })
  .listen("0.0.0.0:8080")
  .await?;

  Ok(())
}
```

### Streamed Response

```rust
use std::time::Duration;

use uhttp::*;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
  uhttp::http1::create_server(|req, mut res| async move {
    // Send the headers before sending the body chunks
    res.write_head(uhttp::StatusCode::OK).await?;

    for i in 0..10 {
      res.write_all(format!("{}", i).as_bytes()).await?;
      tokio::time::sleep(Duration::from_millis(1000)).await;
    }

    Ok(())
  })
  .listen("0.0.0.0:8080")
  .await?;

  Ok(())
}
```

### Static File Server

```rust
use std::path::PathBuf;

use uhttp;
use uhttp::file_server::EtagStrategy;
use uhttp::file_server::FileServerOptions;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
  // Change this to the directory where the files live
  let static_files_dir: PathBuf = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("static");

  uhttp::http1::create_server(uhttp::file_server::create(FileServerOptions {
    dir:static_files_dir,
    // JIT Compression
    compress: true,
    // ETag to prevent client from making multiple requests
    etag: ETagStrategy::LastModified,
  }))
  .listen("0.0.0.0:8080")
  .await?;

  Ok(())
}
```

### Read / Write JSON Payload

```rust
use serde::Deserialize;
use serde::Serialize;
use uhttp::*;

#[derive(Debug, Serialize, Deserialize)]
pub struct BodyJson {
  pub message: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
  uhttp::http1::create_server(|mut req, mut res| async move {
    // Parse incoming JSON body
    let body = uhttp::body::json::<BodyJson>(&mut req.body()).await?;

    // Serialize response body
    let result = serde_json::to_vec(&body)?;

    // Respond with serialized body
    res.write_all(&result).await?;
    Ok(())
  })
  .listen("0.0.0.0:8080")
  .await?;

  Ok(())
}
```

### Router / router

```rust
use uhttp::*;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
  let mut app = uhttp::router::Router::new();

  app.get("/foo", |_req, mut res| async move {
    res.write(b"foo\n").await?;
    Ok(())
  });

  app.post("/bar", |_req, mut res| async move {
    res.write(b"bar\n").await?;
    Ok(())
  });

  // Example of a URL parameter
  app.get("/fizz/:buzz", |req, mut res| async move {
    res.write(b"fizz\n").await?;

    let Some(buzz) = req.url_param("buzz") else {
      res.write_head(uhttp::StatusCode::BAD_REQUEST).await?;
      return Ok(());
    };

    res.write_all(format!("Param: {}", buzz).as_bytes()).await?;
    Ok(())
  });

  // Can be used to serve static assets
  app.not_found(|_req, mut res| async move {
    res.write(b"Not found route").await?;
    Ok(())
  });

  uhttp::http1::create_server(app.handler())
    .listen("0.0.0.0:8080")
    .await?;

  Ok(())
}
```

### HTTP/2

```rust
use uhttp;
use uhttp::AsyncWriteExt;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
  let cert_path = PathBuf::from(std::env::var("SSL_CERT_PATH").expect("Missing SSL_CERT_PATH env var"));
  let key_path = PathBuf::from(std::env::var("SSL_KEY_PATH").expect("Missing SSL_KEY_PATH env var"));

  uhttp::http2::create_server(
    uhttp::http2::Http2ServerOptions {
      cert_path: Some(cert_path),
      key_path: Some(key_path),
    },
    |_req, mut res| async move {
      res.header().add("Content-Type", "text/html").await?;
      res.write_all(b"<body>Hello World!</body>").await?;
      return Ok(());
    },
  )
  .listen("0.0.0.0:8080")
  .await?;

  Ok(())
}

```

### Websockets

\_Note: this can also be used from the router

```rust
use std::time::Duration;

use uhttp;
use uhttp::websocket::WebSocket;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
  uhttp::http1::create_server(move |req, res| async move {
      let (mut socket_sender, mut socket_reciever) = WebSocket::upgrade(req, res).await?;

      tokio::task::spawn(async move {
        loop {
          if socket_sender.send_text("Hello").await.is_err() {
            break;
          };
          tokio::time::sleep(Duration::from_millis(1000)).await;
        }
      });

      tokio::task::spawn(async move {
        while let Some(Ok(msg)) = socket_reciever.next().await {
          println!("GOT: {:?}", msg);
        }
      });

      Ok(())
    })
    .listen("0.0.0.0:8080")
    .await?;

  Ok(())
}
```

# Benchmarks

- AMD 7950X
- 100GB RAM

## Go

```
Summary:
  Success rate:	100.00%
  Total:	448.1539 ms
  Slowest:	10.6163 ms
  Fastest:	0.0193 ms
  Average:	0.4393 ms
  Requests/sec:	223137.6305

  Total data:	1.14 MiB
  Size/request:	12 B
  Size/sec:	2.55 MiB

Response time histogram:
   0.019 ms [1]     |
   1.079 ms [90055] |■■■■■■■■■■■■■■■■■■■■■■■■■■■■■■■■
   2.139 ms [8566]  |■■■
   3.198 ms [1106]  |
   4.258 ms [184]   |
   5.318 ms [38]    |
   6.378 ms [13]    |
   7.437 ms [12]    |
   8.497 ms [16]    |
   9.557 ms [5]     |
  10.616 ms [4]     |

Response time distribution:
  10.00% in 0.0869 ms
  25.00% in 0.1345 ms
  50.00% in 0.2533 ms
  75.00% in 0.4971 ms
  90.00% in 1.0753 ms
  95.00% in 1.5735 ms
  99.00% in 2.3279 ms
  99.90% in 4.1377 ms
  99.99% in 8.2329 ms


Details (average, fastest, slowest):
  DNS+dialup:	0.3273 ms, 0.1281 ms, 0.5560 ms
  DNS-lookup:	0.0189 ms, 0.0010 ms, 0.1469 ms

Status code distribution:
  [200] 100000 responses
```

## uHTTP

```
Summary:
  Success rate:	100.00%
  Total:	208.0683 ms
  Slowest:	3.4607 ms
  Fastest:	0.0240 ms
  Average:	0.1963 ms
  Requests/sec:	480611.3014

  Total data:	1.14 MiB
  Size/request:	12 B
  Size/sec:	5.50 MiB

Response time histogram:
  0.024 ms [1]     |
  0.368 ms [95935] |■■■■■■■■■■■■■■■■■■■■■■■■■■■■■■■■
  0.711 ms [2804]  |
  1.055 ms [857]   |
  1.399 ms [148]   |
  1.742 ms [42]    |
  2.086 ms [45]    |
  2.430 ms [83]    |
  2.773 ms [11]    |
  3.117 ms [21]    |
  3.461 ms [53]    |

Response time distribution:
  10.00% in 0.1046 ms
  25.00% in 0.1325 ms
  50.00% in 0.1700 ms
  75.00% in 0.2183 ms
  90.00% in 0.2819 ms
  95.00% in 0.3441 ms
  99.00% in 0.7954 ms
  99.90% in 2.4065 ms
  99.99% in 3.4094 ms


Details (average, fastest, slowest):
  DNS+dialup:	0.2823 ms, 0.0839 ms, 0.5242 ms
  DNS-lookup:	0.0164 ms, 0.0010 ms, 0.1330 ms

Status code distribution:
  [200] 100000 responses
```
