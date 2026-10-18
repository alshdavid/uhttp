/*
  Test with:
    curl http://localhost:8080
*/

use std::path::PathBuf;

use include_dir::Dir;
use include_dir::include_dir;
use uhttp::Server;
use uhttp::file_server::AsyncIncludeDirFileSystem;
use uhttp::file_server::FileServer;
use uhttp::file_server::SourceMapMode;
use uhttp::file_server::TranspileOptions;

static STATIC_DIR: Dir<'static> = include_dir!("$CARGO_MANIFEST_DIR/static");

#[tokio::main]
async fn main() -> anyhow::Result<()> {
  // Change this to the directory where the files live
  let static_files_dir: PathBuf = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("static");

  Server::builder()
    .handler(
      FileServer::builder()
        .root_dir(static_files_dir)
        .compress(true)
        .file_system(AsyncIncludeDirFileSystem::new(STATIC_DIR.clone()))
        .implicit_extensions(true)
        .cache(Default::default())
        .transpile(TranspileOptions {
          source_maps: SourceMapMode::Linked,
          minify: true,
          mangle: true,
          compress: true,
          worker_threads: None,
        })
        .build()?,
    )
    .listen("0.0.0.0:8080")
    .await?;

  Ok(())
}
