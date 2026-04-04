/*
  Test with:
    curl -H "Content-Type: application/json" -d '{ "message": "Hello World" }' http://localhost:8080
*/
use serde::Deserialize;
use serde::Serialize;
use uhttp::*;

#[derive(Debug, Serialize, Deserialize)]
pub struct BodyJson {
  pub message: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
  uhttp::http1::create_server("0.0.0.0:8080", |mut req, mut res| async move {
    // Parse incoming JSON body
    let body = uhttp::body::json::<BodyJson>(&mut req.body()).await?;

    // Serialize response body
    let result = serde_json::to_vec(&body)?;

    // Respond with serialized body
    res.write_all(&result).await?;
    Ok(())
  })
  .await
}
