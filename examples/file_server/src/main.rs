/*
  Test with:
    curl http://localhost:8080
*/

use std::path::PathBuf;

use uhttp;
use uhttp::file_server::ETagStrategy;
use uhttp::file_server::FileServerOptions;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
  // Change this to the directory where the files live
  let static_files_dir: PathBuf = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("static");

  uhttp::http1::create_server(uhttp::file_server::create(FileServerOptions {
    dir: static_files_dir,
    compress: true,
    etag: ETagStrategy::LastModified,
  }))
  .listen("0.0.0.0:8080")
  .await?;

  Ok(())
}
