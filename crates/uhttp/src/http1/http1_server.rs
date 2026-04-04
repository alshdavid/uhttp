use std::future::Future;

use super::Http1Server;
use super::Request;
use super::Response;

pub fn create_server<F, Fut>(handle_func: F) -> Http1Server<F, Fut>
where
  F: 'static + Send + Sync + Fn(Request, Response) -> Fut,
  Fut: 'static + Send + Future<Output = anyhow::Result<()>>,
{
  Http1Server::new(handle_func)
}
