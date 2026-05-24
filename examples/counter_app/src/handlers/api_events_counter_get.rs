use uhttp::AsyncWriteExt;

use crate::context::Context;

// Event Source that emits when the value is updated
pub async fn api_events_counter_get(
  _req: uhttp::Request,
  mut res: uhttp::Response,
  Context { counter_service }: Context,
) -> uhttp::Result<()> {
  res
    .header()
    .add("Content-Type", "text/event-stream")
    .await?;

  res.header().add("Transfer-Encoding", "chunked").await?;
  res.write_head(uhttp::StatusCode::OK).await?;

  // Send initial value with subscription
  res
    .write_all(format!("data: {}\n\n", counter_service.get()).as_bytes())
    .await?;

  // Listen for updates
  let mut rx = counter_service.subsribe().await;
  while let Some(_) = rx.recv().await {
    let msg = format!("data: {}\n\n", counter_service.get());
    res.write_all(msg.as_bytes()).await?;
  }

  Ok(())
}
