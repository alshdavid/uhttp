/*
  Test with:
    curl http://localhost:8080
*/
mod counter_service;

use std::path::PathBuf;

use uhttp;
use uhttp::AsyncWriteExt;

use crate::counter_service::CoutnerService;

static CARGO_MANIFEST_DIR: &str = env!("CARGO_MANIFEST_DIR");

#[tokio::main]
async fn main() -> anyhow::Result<()> {
  println!("{}", CARGO_MANIFEST_DIR);

  let mut app = uhttp::router::Router::new();

  let counter_service = CoutnerService::new();

  // Get current value
  app.get("/api/counter", {
    let counter_service = counter_service.clone();
    move |_req, mut res| {
      let counter_service = counter_service.clone();
      async move {
        let counter_value = counter_service.get();

        let json = serde_json::json!({
          "value": counter_value
        });

        let msg = serde_json::to_string_pretty(&json)
          .map_err(|_| uhttp::Error::generic("Unable to serialize message"))?;

        res.write_all(msg.as_bytes()).await?;

        Ok(())
      }
    }
  });

  // Event Source that emits when the value is updated
  app.get("/api/events/counter", {
    let counter_service = counter_service.clone();
    move |_req, mut res| {
      let counter_service = counter_service.clone();
      async move {
        res
          .header()
          .add("Content-Type", "text/event-stream")
          .await?;
        
        res.header().add("Transfer-Encoding", "chunked").await?;
        res.write_head(uhttp::StatusCode::OK).await?;

        // Send initial value with subscription
        res
          .write_all(format!("data: {}\n\n", counter_service.get()).as_bytes())
          .await?;

        // Listen for updates
        let mut rx = counter_service.subsribe().await;
        while let Some(_) = rx.recv().await {
          res.write_all(b"data: updated\n\n").await?;
          res
            .write_all(format!("data: {}\n\n", counter_service.get()).as_bytes())
            .await?;
        }

        Ok(())
      }
    }
  });

  app.post("/api/counter/increment", {
    let counter_service = counter_service.clone();
    move |_req, res| {
      let counter_service = counter_service.clone();
      async move {
        counter_service.increment().await;
        res.write_head(uhttp::StatusCode::NO_CONTENT).await?;
        Ok(())
      }
    }
  });

  app.post("/api/counter/decrement", {
    let counter_service = counter_service.clone();
    move |_req, res| {
      let counter_service = counter_service.clone();
      async move {
        counter_service.decrement().await;
        res.write_head(uhttp::StatusCode::NO_CONTENT).await?;
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
