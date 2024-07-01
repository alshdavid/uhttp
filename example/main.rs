use std::io;
use std::io::Write;

use uhttp::body_parser;
use uhttp::Server;

#[tokio::main]
async fn main() -> io::Result<()> {
  let server = Server::new(|mut req, mut res| {
    println!("hi");
    // let value = req.headers.get("Accept-Encoding");
    dbg!(&req);
    // println!("{:?}", value);
    // println!("{:?}", req.url);

    // if req.url == "/" {
    //   res.headers().set("Access-Control-Allow-Origin", "*");
    //   res.headers().set("Content-Type", "text/html");
    //   res.write_header(200)?;

    //   let body = body_parser::utf8(&mut req.body)?;
    //   println!("{}", body);

    //   write!(res, "<body>Hello world</body>")?;
    // }

    Ok(())
  });

  server.listen("0.0.0.0:3000").await?;

  Ok(())
}
