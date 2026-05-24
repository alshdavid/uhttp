use crate::context::Context;

pub async fn api_events_counter_increment_post(
  _req: uhttp::Request,
  res: uhttp::Response,
  Context { counter_service }: Context,
) -> uhttp::Result<()> {
  counter_service.increment().await;
  res.write_head(uhttp::StatusCode::NO_CONTENT).await?;
  Ok(())
}
