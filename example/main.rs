use std::io::Write;
use std::io;

use uhttp::Server;

fn main() -> io::Result<()> {
  let server = Server::new(|req, mut res| {
    let value = req.headers.get("Accept-Encoding");
    println!("{:?}", value);
    println!("{:?}", req.url);

    if req.url == "/" {
      res.headers().add("Access-Control-Allow-Origin", "*");
      res.headers().add("Content-Type", "text/html");
      res.write_header(200)?;

      write!(res, "<body>Hello world</body>")?;
    }

    Ok(())
  });


  server.listen("0.0.0.0:3000")?;
  
  Ok(())
}
