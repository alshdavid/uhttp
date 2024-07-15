use std::io;
use std::io::Write;

use uhttp::sync::{body_parser, Server};

fn main() -> io::Result<()> {
  let server = Server::new(|mut req, mut res| {
    res.headers().set("Content-Type", "text/plain");
    res.write_header(200)?;

    // body_parser::utf8(&mut req.body)?;
    res.write_all(b"Hello, World!")?;

    Ok(())
  });

  server.listen("0.0.0.0:8080")?;

  Ok(())
}
