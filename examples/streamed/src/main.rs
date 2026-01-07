/*
  Test with:
    curl -N http://localhost:8080
*/
use std::time::Duration;

use tokio::io::AsyncWriteExt;
use tokio::time::sleep;
use uhttp;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
  uhttp::http1::create_server("0.0.0.0:8080", |req, mut res| async move {
    println!("{}", req.uri());

    res.write_head(uhttp::StatusCode::OK).await?;

    res.write_all(b"1\n").await?;

    sleep(Duration::from_millis(1000)).await;
    res.write_all(b"2\n").await?;

    sleep(Duration::from_millis(1000)).await;
    res.write_all(b"3\n").await?;
    Ok(())
  })
  .await
}
