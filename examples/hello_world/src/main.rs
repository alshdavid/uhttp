use std::io;
use std::io::Write;

use uhttp::c;
use uhttp::http_1::*;
use uhttp::*;

fn main() -> io::Result<()> {
  Server::new(handler).listen("0.0.0.0:8080")
}

fn handler(
  _req: Request,
  mut res: Response,
) -> io::Result<()> {
  res.header(c::headers::CONTENT_TYPE, c::content_type::TEXT_PLAIN);

  res.write(b"Hello World!")?;
  res.write_header(c::status::OK)
}
