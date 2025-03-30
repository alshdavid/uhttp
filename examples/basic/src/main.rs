#![allow(clippy::unused_io_amount)]

use tokio::io::AsyncWriteExt;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
  uhttp::http1::create_server(
    "0.0.0.0:8080",
    |_req, res| Box::pin(async move {
      res.write(b"hello world\n").await?;
      Ok(())
    })).await?;

  Ok(())
}


