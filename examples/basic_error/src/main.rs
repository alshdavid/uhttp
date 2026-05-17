/*
  Test with:
    curl http://localhost:8080
*/
use uhttp;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
  uhttp::http1::create_server(|_req, mut _res| async move {
    println!("Throwing in handler");
    return Err(anyhow::anyhow!("Something bad happened"));
  })
  .listen("0.0.0.0:8080")
  .await
}
