#![allow(clippy::unused_io_amount)]

use tokio::io::AsyncReadExt;
use tokio::io::AsyncWriteExt;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
  uhttp::http1::create_server(|mut req, res| {
    Box::pin(async move {
      let mut data = String::new();
      req.body().read_to_string(&mut data).await?;

      println!("req: {}", data);
      res.write(format!("hello world\n").as_bytes()).await?;
      Ok(())
    })
  })
  .listen("0.0.0.0:8080")
  .await
}
