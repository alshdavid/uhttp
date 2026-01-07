/*
  Test with:
    curl -H "Content-Type: application/json" -d '{ "message": "Hello World" }' http://localhost:8080
*/
use serde::Deserialize;
use tokio::io::AsyncWriteExt;
use uhttp;

#[derive(Debug, Deserialize)]
pub struct BodyJson {
  pub message: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
  uhttp::http1::create_server("0.0.0.0:8080", |mut req, mut res| async move {
    println!("{}", req.uri());

    let body = uhttp::body::json::<BodyJson>(&mut req.body()).await?;
    dbg!(&body);

    res.write_all(b"Ok\n").await?;
    Ok(())
  })
  .await
}
