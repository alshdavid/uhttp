use std::io;

use uhttp::http1::Server;
use uhttp::HttpWriter;
use uhttp::Request;
use uhttp::Response;

#[tokio::main]
async fn main() -> io::Result<()> {
  Server::new(handler).listen("0.0.0.0:8080").await
}

async fn handler(mut _req: Request, mut res: Response) -> io::Result<()> {
  res.write_all(b"Hello World!").await?;
  res.write_header(200).await
}