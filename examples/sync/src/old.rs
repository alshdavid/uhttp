use std::io;
use std::io::Write;

// use uhttp::sync::body_parser;
use uhttp::sync::Server;

fn main() -> io::Result<()> {
  let server = Server::new(|mut req, mut res| {
    res.headers().set("Access-Control-Allow-Origin", "*");
    res.headers().set("Content-Type", "text/html");
    res.write_header(200)?;

    // let body = body_parser::utf8(&mut req.body)?;
    // println!("BODY: {}", body);

    write!(res, "<body>Hello world</body>")?;
    Ok(())
  });

  server.listen("0.0.0.0:8080")?;

  Ok(())
}
