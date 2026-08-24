/*
  Test with:
    curl http://localhost:8080
*/
use uhttp::AsyncWriteExt;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
  uhttp::http1::create_server(|_req, mut res| async move {
    res.header().add("Content-Type", "application/json").await?;

    let body = serde_json::json!({ "message": "hello world" });
    let result = serde_json::to_vec(&body)?;

    res.write_all(&result).await?;
    Ok(())
  })
  .listen("0.0.0.0:8080")
  .await
}
