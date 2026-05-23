/*
  Test with:
    curl https://localhost:8080
*/
use std::path::PathBuf;

use anyhow::Context;
use uhttp;
use uhttp::AsyncWriteExt;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
  let cert_path =
    PathBuf::from(std::env::var("SSL_CERT_PATH").context("Missing SSL_CERT_PATH env var")?);
  let key_path =
    PathBuf::from(std::env::var("SSL_KEY_PATH").context("Missing SSL_KEY_PATH env var")?);

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
