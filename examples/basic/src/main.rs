/*
  Test with:
    curl http://localhost:8080
*/
use uhttp;
use uhttp::AsyncWriteExt;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
  uhttp::http1::create_server("0.0.0.0:8080", |_req, mut res| async move {
    res.header().add("Content-Type", "text/html").await?;
    res.write_all(b"<body>Hello World!</body>").await?;
    return Ok(());
  })
  .await
}
