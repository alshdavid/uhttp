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

### Standard Response

```rust
use uhttp::*;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
  uhttp::http1::create_server("0.0.0.0:8080", |req, mut res| async move {
    res.header().add("Content-Type", "text/html")
    res.write_all(b"<body>hello world</body>").await
  }).await
}
```

### Streamed Response

```rust
use uhttp::*;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
  uhttp::http1::create_server("0.0.0.0:8080", |req, mut res| async move {
    // Send the headers before sending the body chunks
    res.write_head(uhttp::StatusCode::OK).await?;
    
    for i in 0..10 {
      res.write_all(format!("{}", i).as_bytes()).await?;
      tokio::time::sleep(Duration::from_millis(1000)).await;
    }
    
    Ok(())
  }).await
}
```

### Read JSON Payload

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
  uhttp::http1::create_server("0.0.0.0:8080", |mut req, mut res| async move {
    // Parse incoming JSON body
    let body = uhttp::body::json::<BodyJson>(&mut req.body()).await?;

    // Serialize response body
    let result = serde_json::to_vec(&body)?;
    
    // Respond with serialized body
    res.write_all(&result).await?;
    Ok(())
  })
  .await
}

```

## WIP

### Server

```rust
use uhttp::*;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
  uhttp::http1::create_server(|req, mut res| async move {
    res.header().add("Content-Type", "text/html")
    res.write_all(b"<body>hello world</body>").await
  })
  .listen("0.0.0.0:8080")
  .await
}
```

### MUX

```rust
use uhttp::*;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
  let mut app = uhttp::mux::Router();

  app.get("/foo", |req, mut res| async move  {
    res.write(b"bar\n").await
  })

  app.post("/bar", |req, mut res| async move {
    res.write(b"foo\n").await
  })

  uhttp::http1::create_server(app.handler())
    .listen("0.0.0.0:8080")
    .await
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
