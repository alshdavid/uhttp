use std::sync::Arc;

use crate::services::counter_service::CounterService;

#[derive(Clone)]
pub struct Context {
  pub counter_service: Arc<CounterService>,
}
