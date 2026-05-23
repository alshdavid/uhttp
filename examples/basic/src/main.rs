/*
  Test with:
    curl http://localhost:8080
*/
use uhttp::{self, StatusCode};
use uhttp::AsyncWriteExt;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
  uhttp::http1::create_server(|_req, mut res| async move {
    // res.header().add("Content-Type", "text/html").await?;
    res.write_all(b"<body>Hello World!</body>").await?;
    res.write_head(StatusCode::OK).await?;
    return Ok(());
  })
  .listen("0.0.0.0:8080")
  .await?;

  Ok(())
}
