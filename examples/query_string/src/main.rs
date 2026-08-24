/*
  Test with:
    curl http://localhost:8080/?hello=world
*/
use std::collections::HashMap;

use uhttp::AsyncWriteExt;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
  uhttp::http1::create_server(|req, mut res| async move {
    let query = req.parse_query::<HashMap<String, String>>()?;

    res.header().add("Content-Type", "text/html").await?;
    res
      .write_all(format!("<body>{:?}</body>", query).as_bytes())
      .await?;
    Ok(())
  })
  .listen("0.0.0.0:8080")
  .await?;

  Ok(())
}
