use std::io;

use uhttp::http1::Server;
use uhttp::HttpWriter;
use uhttp::Request;
use uhttp::Response;

#[tokio::main]
async fn main() -> io::Result<()> {
  Server::new(handler).listen("0.0.0.0:8080").await
}

async fn handler(mut req: Request, mut res: Response) -> io::Result<()> {
  if req.method == "GET" && req.url == "/" {
    return res.write_all(b"Hello World!").await
  }

  if req.method == "POST" && req.url == "/api/echo" {
    let bytes = uhttp::utils::body::bytes(&mut req).await?;
    return res.write_all(&bytes).await
  }

  res.write_header(404).await
}