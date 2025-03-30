#![allow(clippy::unused_io_amount)]

use tokio::io::AsyncWriteExt;
use uhttp::HandlerFunc;

fn handler_hello_world() -> impl HandlerFunc {
  move |_req, res| {
    Box::pin(async move {
      res.write(b"hello world\n").await?;
      Ok(())
    })
  }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
  uhttp::http1::create_server(handler_hello_world())
    .listen("0.0.0.0:8080")
    .await?;

  Ok(())
}
