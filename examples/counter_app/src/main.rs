/*
  Test with:
    curl http://localhost:8080
*/
mod counter_service;

use std::path::PathBuf;
use std::sync::Arc;

use uhttp::AsyncWriteExt;
use uhttp::file_server;
use uhttp::file_server::ETagStrategy;
use uhttp::file_server::FileServerOptions;
use uhttp::{self};

use crate::counter_service::CounterService;

static CARGO_MANIFEST_DIR: &str = env!("CARGO_MANIFEST_DIR");

#[derive(Clone)]
struct Context {
  counter_service: Arc<CounterService>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
  let mut app = uhttp::router::Router::new(Context {
    counter_service: Arc::new(CounterService::new()),
  });

  // Get current value
  app.get(
    "/api/counter",
    move |_req, mut res, Context { counter_service }: Context| async move {
      let counter_value = counter_service.get();

      let json = serde_json::json!({
        "value": counter_value
      });

      let msg = serde_json::to_string_pretty(&json)
        .map_err(|_| uhttp::Error::generic("Unable to serialize message"))?;

      res.write_all(msg.as_bytes()).await?;

      Ok(())
    },
  );

  // Event Source that emits when the value is updated
  app.get(
    "/api/events/counter",
    move |_req, mut res, Context { counter_service }: Context| async move {
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
        let msg = format!("data: {}\n\n", counter_service.get());
        res.write_all(msg.as_bytes()).await?;
      }

      Ok(())
    },
  );

  app.post(
    "/api/counter/increment",
    move |_req, res, Context { counter_service }: Context| async move {
      counter_service.increment().await;
      res.write_head(uhttp::StatusCode::NO_CONTENT).await?;
      Ok(())
    },
  );

  app.post("/api/counter/decrement", {
    move |_req, res, Context { counter_service }: Context| async move {
      counter_service.decrement().await;
      res.write_head(uhttp::StatusCode::NO_CONTENT).await?;
      Ok(())
    }
  });

  app.without_context().get(
    "/*",
    file_server::create(FileServerOptions {
      dir: PathBuf::from(CARGO_MANIFEST_DIR).join("static"),
      compress: false,
      etag: ETagStrategy::LastModified,
    }),
  );

  uhttp::http1::create_server(app.handler())
    .listen("0.0.0.0:8080")
    .await?;

  Ok(())
}
