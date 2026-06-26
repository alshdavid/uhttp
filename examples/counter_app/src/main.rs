/*
  Test with:
    curl http://localhost:8080
*/
mod context;
mod handlers;
mod services;

use std::path::PathBuf;
use std::sync::Arc;

use uhttp::file_server;
use uhttp::file_server::ETagStrategy;
use uhttp::file_server::FileServerOptions;
use uhttp::{self};

use crate::context::Context;
use crate::services::counter_service::CounterService;

static CARGO_MANIFEST_DIR: &str = env!("CARGO_MANIFEST_DIR");

#[rustfmt::skip]
#[tokio::main]
async fn main() -> anyhow::Result<()> {
  let ctx = Context {
    counter_service: Arc::new(CounterService::new()),
  };

  let mut app = uhttp::router::Router::new(ctx);

  app.with_all(uhttp::middleware::logger_default);

  app.get("/api/counter", handlers::api_counter_get);
  app.get("/api/events/counter", handlers::api_events_counter_get);
  app.post("/api/counter/increment", handlers::api_events_counter_increment_post);
  app.post("/api/counter/decrement", handlers::api_events_counter_decrement_post);

  app
    .without_context()
    .get("/*", file_server::create(FileServerOptions {
      dir: PathBuf::from(CARGO_MANIFEST_DIR).join("static"),
      compress: false,
      etag: ETagStrategy::LastModified,
      fallback_route: Default::default(),
      fallback_status: Default::default(),
    }));

  uhttp::http1::create_server(app.handler())
    .listen("0.0.0.0:8080")
    .await?;

  Ok(())
}
