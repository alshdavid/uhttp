use serde::Serialize;
use uhttp::AsyncWriteExt;

use crate::context::Context;

#[derive(Debug, Serialize)]
struct ApiCounterGetResponse {
  value: isize,
}

// Get current value
pub async fn api_counter_get(
  _req: uhttp::Request,
  mut res: uhttp::Response,
  Context { counter_service }: Context,
) -> uhttp::Result<()> {
  let counter_value = counter_service.get();

  let json = serde_json::to_vec(&ApiCounterGetResponse {
    value: counter_value,
  })?;

  let msg = serde_json::to_string_pretty(&json)?;

  res.write_all(msg.as_bytes()).await?;

  Ok(())
}
