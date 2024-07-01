use std::io::Write;
use std::io;

use uhttp::sync::Server;
use uhttp::body_parser;

fn main() -> io::Result<()> {
  let server = Server::new(|mut req, mut res| {
    // let value = req.headers.get("Accept-Encoding");
    dbg!(&req);
    // println!("{:?}", value);
    // println!("{:?}", req.url);

    if req.url == "/" {
      res.headers().set("Access-Control-Allow-Origin", "*");
      res.headers().set("Content-Type", "text/html");
      res.write_header(200)?;

      
      let body = body_parser::utf8(&mut req.body)?;
      println!("{}", body);

      write!(res, "<body>Hello world</body>")?;
    }

    Ok(())
  });


  server.listen("0.0.0.0:3000")?;
  
  Ok(())
}
