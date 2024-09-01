use std::io;

use uhttp::c;
use uhttp::http1::*;
use uhttp::*;

#[tokio::main]
async fn main() -> io::Result<()> {
  Server::new(handler).listen("0.0.0.0:8080").await
}

async fn handler(
  mut _req: Request,
  mut res: Response,
) -> io::Result<()> {
  res.header(c::headers::CONTENT_TYPE, c::content_type::TEXT_PLAIN);

  res.write_all(b"Hello World!").await?;
  res.write_header(c::status::OK)
}
