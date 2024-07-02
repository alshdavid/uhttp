use std::io;
use std::io::Write;

use uhttp::sync::Server;

fn main() -> io::Result<()> {
  let server = Server::new(|req, mut res| {
    if req.url == "/" {
      res.headers().set("Content-Type", "text/plain");
      res.write_header(200)?;

      res.write_all(b"Hello, World!")?;
    }

    Ok(())
  });

  server.listen("0.0.0.0:8080")?;

  Ok(())
}
