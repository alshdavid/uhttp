use std::io;
use std::path::PathBuf;

use tokio::fs;

use uhttp::http1::Server;
use uhttp::HttpWriter;
use uhttp::Request;
use uhttp::Response;

const CARGO_HOME: &str = env!("CARGO_MANIFEST_DIR");

#[tokio::main]
async fn main() -> io::Result<()> {
  Server::new(handler).listen("0.0.0.0:8080").await
}

async fn handler(_req: Request, mut res: Response) -> io::Result<()> {
  let index_file = PathBuf::from(CARGO_HOME).join("index.html");
  let bytes = fs::read(&index_file).await?;
  res.write_all(&bytes).await?;
  res.write_header(200).await
}
