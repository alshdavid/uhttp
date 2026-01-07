#![allow(clippy::unused_io_amount)]

use std::time::Duration;

use tokio::io::AsyncWriteExt;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
  uhttp::http1::http1_server(
    format!("{}:{}", "0.0.0.0", "8080"),
    |req, mut res| async move {
      println!("{}", req.uri());

      res.write_all("1\n".as_bytes()).await?;

      tokio::time::sleep(Duration::from_millis(100)).await;
      res.write_all("2\n".as_bytes()).await?;
      res.write_head(http::StatusCode::OK).await?;

      res.end().await?;

      tokio::time::sleep(Duration::from_millis(100)).await;
      res.write_all("3\n".as_bytes()).await?;

      Ok(())
    },
  )
  .await?;

  Ok(())
}
