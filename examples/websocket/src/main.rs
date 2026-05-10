/*
  Test with:
    websocat ws://localhost:8080
*/
use std::time::Duration;

use uhttp;
use uhttp::websocket::WebSocket;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
  uhttp::http1::create_server(move |req, res| async move {
    let (mut socket_sender, mut socket_reciever) = WebSocket::upgrade(req, res).await?;

    tokio::task::spawn(async move {
      loop {
        if socket_sender.send_text("Hello").await.is_err() {
          break;
        };
        tokio::time::sleep(Duration::from_millis(1000)).await;
      }
    });

    tokio::task::spawn(async move {
      while let Some(Ok(msg)) = socket_reciever.next().await {
        println!("GOT: {:?}", msg);
      }
    });

    Ok(())
  })
  .listen("0.0.0.0:8080")
  .await?;

  Ok(())
}
