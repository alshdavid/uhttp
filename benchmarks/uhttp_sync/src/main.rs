use std::io;
use std::io::Write;

use uhttp::sync::Server;

fn main() -> io::Result<()> {
  let server = Server::new(|_req, mut res| {
    let data = b"hello world";
    res.headers().set("Content-Type", "text/plain");
    res.headers().set("Content-Length", format!("{}", data.len()));
    res.write_header(200)?;

    res.write_all(data)?;
    res.end()?;
    Ok(())
  });

  server.listen("0.0.0.0:8080")?;

  Ok(())
}
