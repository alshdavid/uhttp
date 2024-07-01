use std::io;
use std::time::Duration;

use uhttp::body_parser;
use uhttp::Server;

#[tokio::main]
async fn main() -> io::Result<()> {
  let server = Server::new(|mut req, mut res| async move {
    res.headers().set("Access-Control-Allow-Origin", "*");
    res.headers().set("Content-Type", "text/html");
    // tokio::time::sleep(Duration::from_secs(1)).await;
    res.write_header(200).await?;

    let body = body_parser::bytes(&mut req.body).await?;

    println!("BODY: {:?}", body);

    res.write(b"<body>Hello world</body>").await?;
    Ok(())
  });

  server.listen("0.0.0.0:3000").await?;

  Ok(())
}
