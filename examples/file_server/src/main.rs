use std::path::PathBuf;

/*
  Test with:
    curl http://localhost:8080
*/
use uhttp;
use uhttp::file_server::EtagStrategy;
use uhttp::file_server::FileServerOptions;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
  // Change this to the directory where the files live
  let cargo_toml_dir: &str = env!("CARGO_MANIFEST_DIR");

  uhttp::http1::create_server(uhttp::file_server::handler(FileServerOptions {
    base_dir: PathBuf::from(cargo_toml_dir).join("static"),
    compress: true,
    etag: EtagStrategy::LastModified,
  }))
  .listen("0.0.0.0:8080")
  .await?;

  Ok(())
}
