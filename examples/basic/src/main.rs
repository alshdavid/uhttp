/*
  Test with:
    curl http://localhost:8080
*/
use uhttp::AsyncWriteExt;
use uhttp::StatusCode;
use uhttp::{self};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
  uhttp::http1::create_server(|_req, mut res| async move {
    // res.header().add("Content-Type", "text/html").await?;
    res.write_head(StatusCode::OK).await?;
    res.write_all(b"<body>Hello World!</body>").await?;
    res.flush().await?;
    res.write_all(b"<body>Hello World!</body>").await?;

    res.write_all(b"<body>Hello World!</body>").await?;

    return Ok(());
  })
  .listen("0.0.0.0:8080")
  .await?;

  Ok(())
}
