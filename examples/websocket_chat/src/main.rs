/*
  Test with:
    curl http://localhost:8080
*/
mod chat_service;

use std::path::PathBuf;
use std::sync::Arc;

use tokio::sync::Mutex;
use uhttp;
use uhttp::AsyncWriteExt;
use uhttp::websocket::WebSocket;

use crate::chat_service::ChatService;

static CARGO_MANIFEST_DIR: &str = env!("CARGO_MANIFEST_DIR");

#[tokio::main]
async fn main() -> anyhow::Result<()> {
  let mut app = uhttp::router::Router::new();

  let chat_service = Arc::new(Mutex::new(ChatService::new()));

  // Event Source that emits when the value is updated
  app.any("/api/ws", {
    let chat_service = Arc::clone(&chat_service);
    move |req, res| {
      let chat_service = Arc::clone(&chat_service);
      async move {
        let (mut socket_sender, mut socket_reciever) = WebSocket::upgrade(req, res).await?;

        // Send action
        tokio::task::spawn({
          let chat_service = Arc::clone(&chat_service);
          async move {
            let mut chat_service = chat_service.lock().await;

            // Send initial chats as the first message
            let messages = chat_service.get();
            let Ok(msg) = serde_json::to_string_pretty(&messages) else {
              return;
            };

            if socket_sender.send_text(msg).await.is_err() {
              return;
            };

            // Create subscription for subsequent messages
            let mut rx = chat_service.subsribe();
            drop(chat_service);

            // Emit new messages to current socket
            while let Some((author, message)) = rx.recv().await {
              let msg = serde_json::json!([[author, message]]);
              let Ok(msg) = serde_json::to_string(&msg) else {
                break;
              };

              if socket_sender.send_text(msg).await.is_err() {
                break;
              };
            }
          }
        });

        // Recieve action
        tokio::task::spawn({
          let chat_service = Arc::clone(&chat_service);
          async move {
            while let Some(Ok(msg)) = socket_reciever.next_text().await {
              let Ok((author, message)) = serde_json::from_str::<(String, String)>(&msg) else {
                continue;
              };

              let mut chat_service = chat_service.lock().await;
              chat_service.new_message(author, message);
            }
          }
        });

        Ok(())
      }
    }
  });

  // Serve static files from the "static" directory, and return 404 for missing files
  app.not_found(|req, mut res| async move {
    let mut uri = req.uri().path();
    if uri == "/" {
      uri = "/index.html";
    }

    let static_path = PathBuf::from(format!("{}/static{}", CARGO_MANIFEST_DIR, uri));
    let Ok(mut file) = tokio::fs::File::open(static_path).await else {
      res.write_all(b"Not Found").await?;
      res.write_head(uhttp::StatusCode::NOT_FOUND).await?;
      return Ok(());
    };

    res.write_head(uhttp::StatusCode::OK).await?;
    tokio::io::copy(&mut file, &mut res).await?;
    Ok(())
  });

  uhttp::http1::create_server(app.handler())
    .listen("0.0.0.0:8080")
    .await?;

  Ok(())
}
