#![allow(clippy::unused_io_amount)]

use std::time::Duration;

use tokio::io::AsyncWriteExt;
use uhttp::HandlerFunc;

fn handler_chunked() -> impl HandlerFunc {
  move |_req, res| {
    Box::pin(async move {
      res
        .headers_mut()
        .append("Content-Type", "text/html".try_into()?);
      res
        .headers_mut()
        .append("X-Accel-Buffering", "no".try_into()?);
      res
        .headers_mut()
        .append("Content-Type", "text/event-stream".try_into()?);
      res
        .headers_mut()
        .append("Cache-Control", "no-cache".try_into()?);
      res
        .headers_mut()
        .append("Connection", "keep-alive".try_into()?);
      res
        .headers_mut()
        .append("Access-Control-Allow-Origin", "*".try_into()?);

      // Start a thread and asynchronously write to the stream
      tokio::task::spawn({
        let mut res = res.split_writer();
        async move {
          for i in 0..5 {
            res
              .write(format!("data: Message #{}\n\n", i).as_bytes())
              .await
              .unwrap();
            tokio::time::sleep(Duration::from_secs(1)).await;
          }
        }
      });

      Ok(())
    })
  }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
  uhttp::http1::create_server(handler_chunked())
    .listen("0.0.0.0:8080")
    .await
}
