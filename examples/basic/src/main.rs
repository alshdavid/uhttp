/*
  Test with:
    curl http://localhost:8080
*/
use tokio::io::AsyncWriteExt;
use uhttp;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
  uhttp::http1::create_server("0.0.0.0:8080", |req, mut res| async move {
    println!("{}", req.uri());

    res.write_all(b"hello world\n").await?;
    Ok(())
  })
  .await
}
