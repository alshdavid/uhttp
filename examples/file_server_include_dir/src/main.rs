/*
  Test with:
    curl http://localhost:8080
*/

use include_dir::Dir;
use include_dir::include_dir;
use uhttp;
use uhttp::file_server::FileServerIncludeDirOptions;

// Change this to the directory where the files live
static CLIENT_FILES: Dir = include_dir!("$CARGO_MANIFEST_DIR/static");

#[tokio::main]
async fn main() -> anyhow::Result<()> {
  uhttp::http1::create_server(uhttp::file_server::create_include_dir(
    FileServerIncludeDirOptions {
      dir: CLIENT_FILES.clone(),
      compress: true,
    },
  ))
  .listen("0.0.0.0:8080")
  .await?;

  Ok(())
}
