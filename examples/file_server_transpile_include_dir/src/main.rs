/*
  Test with:
    curl http://localhost:8080
*/

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
  Server::builder()
    .handler(
      FileServer::builder()
        .file_system(AsyncIncludeDirFileSystem::new(STATIC_DIR.clone()))
        .root_dir("/")
        .compress(true)
        .implicit_extensions(true)
        .cache(Default::default())
        .transpile(
          TranspileOptions::builder()
            .source_maps(SourceMapMode::Linked)
            .minify(true)
            .mangle(true)
            .compress(true)
            .build(),
        )
        .build()?,
    )
    .listen("0.0.0.0:8080")
    .await?;

  Ok(())
}
