/*
  Test with:
    curl -H "Content-Type: text/plain" -d 'Hello From Client' http://localhost:8080
*/
use tokio::io::AsyncWriteExt;
use uhttp;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
  uhttp::http1::create_server("0.0.0.0:8080", |mut req, mut res| async move {
    println!("{}", req.uri());

    let body = uhttp::body::utf8(&mut req.body()).await?;
    println!("{}", body);

    res.write_all(b"Ok\n").await?;
    Ok(())
  })
  .await
}
