/*
  Test with:
    curl http://localhost:8080
*/

use std::path::PathBuf;

use uhttp::Server;
use uhttp::file_server::FileServer;
use uhttp::file_server::TranspileMode;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
  // Change this to the directory where the files live
  let static_files_dir: PathBuf = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("static");

  Server::builder()
    .handler(
      FileServer::builder()
        .root_dir(static_files_dir)
        .compress(true)
        .transpile(TranspileMode::Development)
        .build()?,
    )
    .listen("0.0.0.0:8080")
    .await?;

  Ok(())
}
