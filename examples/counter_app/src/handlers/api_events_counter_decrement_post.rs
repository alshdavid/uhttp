use crate::context::Context;

pub async fn api_events_counter_decrement_post(
  _req: uhttp::Request,
  res: uhttp::Response,
  Context { counter_service }: Context,
) -> uhttp::Result<()> {
  counter_service.decrement().await;
  res.write_head(uhttp::StatusCode::NO_CONTENT).await?;
  Ok(())
}
