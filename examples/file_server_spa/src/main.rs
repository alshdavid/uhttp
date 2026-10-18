/*
  Test with:
    curl http://localhost:8080
*/

use std::path::PathBuf;

use uhttp::Server;
use uhttp::StatusCode;
use uhttp::file_server::ETagStrategy;
use uhttp::file_server::FileServerOptions;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
  // Change this to the directory where the files live
  let static_files_dir: PathBuf = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("static");

  Server::builder()
    .handler(uhttp::file_server::create(FileServerOptions {
      root_dir: static_files_dir,
      file_system: uhttp::file_server::TokioFileSystem::new(),
      compress: true,
      implicit_extensions: true,
      etag: ETagStrategy::LastModified,
      custom_headers: Default::default(),
      cache: None,
      fallback_route: Some("index.html".to_string()),
      fallback_status: Some(StatusCode::OK),
    })?)
    .listen("0.0.0.0:8080")
    .await?;

  Ok(())
}
