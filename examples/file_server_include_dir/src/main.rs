/*
  Test with:
    curl http://localhost:8080
*/

use dyn_fs::asynch::include_dir::AsyncIncludeDirFileSystem;
use include_dir::Dir;
use include_dir::include_dir;
use uhttp::Server;
use uhttp::file_server::FileServer;

static STATIC_DIR: Dir<'static> = include_dir!("$CARGO_MANIFEST_DIR/static");

#[tokio::main]
async fn main() -> anyhow::Result<()> {
  Server::builder()
    .handler(
      FileServer::builder()
        .file_system(AsyncIncludeDirFileSystem::new(STATIC_DIR.clone()))
        .root_dir("/")
        .compress(true)
        .implicit_extensions(true)
        .build()?,
    )
    .listen("0.0.0.0:8080")
    .await?;

  Ok(())
}
