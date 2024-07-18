use std::io;
use std::io::Write;

use uhttp::http_1::*;
use uhttp::*;

fn main() -> io::Result<()> {
  Server::new(handler).listen("0.0.0.0:8080")
}

fn handler(
  _req: Request,
  mut res: Response,
) -> io::Result<()> {
  res.header("Content-Type", "text/plain");

  res.write(b"Hello World!")?;
  res.write_header(200)
}
