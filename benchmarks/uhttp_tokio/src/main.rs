use std::io;

use uhttp::Server;

#[tokio::main]
async fn main() -> io::Result<()> {
  let server = Server::new(|req, mut res| async move {
    if req.url == "/" {
      res.headers().set("Content-Type", "text/plain");
      res.write_header(200).await?;

      res.write(b"Hello, World!").await?;
    }
    Ok(())
  });

  server.listen("0.0.0.0:8080").await?;

  Ok(())
}

/*
echo "GET http://localhost:8080" | vegeta attack -duration=5s \
  -output=results.bin && cat results.bin | \
  vegeta plot > results.html && \
  rm results.bin
 */
