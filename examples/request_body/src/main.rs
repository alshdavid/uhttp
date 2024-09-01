use std::io;

use uhttp::http1::Server;
use uhttp::Request;
use uhttp::Response;

#[tokio::main]
async fn main() -> io::Result<()> {
  Server::new(handler).listen("0.0.0.0:8080").await
}

async fn handler(mut req: Request, mut res: Response) -> io::Result<()> {
  let body_text = uhttp::utils::body::utf8(&mut req).await?;
  println!("{}", body_text);

  res.write_header(201).await
}
