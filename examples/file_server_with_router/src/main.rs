/*
  Test with:
    curl http://localhost:8080
*/

use std::path::PathBuf;

use uhttp;
use uhttp::AsyncWriteExt;
use uhttp::StatusCode;
use uhttp::file_server::ETagStrategy;
use uhttp::file_server::FileServerOptions;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
  // Change this to the directory where the files live
  let static_files_dir: PathBuf = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("static");

  let mut app = uhttp::router::Router::new_without_context();

  app.get("/api", |_req, mut res, _ctx| async move {
    res.write_all(b"Hello From API").await?;
    res.write_head(StatusCode::OK).await?;
    Ok(())
  });

  app.without_context().get(
    "/*",
    uhttp::file_server::create(FileServerOptions {
      dir: static_files_dir,
      compress: true,
      etag: ETagStrategy::LastModified,
      fallback_route: Default::default(),
      fallback_status: Default::default(),
    }),
  );

  uhttp::http1::create_server(app.handler())
    .listen("0.0.0.0:8080")
    .await?;

  Ok(())
}
