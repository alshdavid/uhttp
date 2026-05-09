use std::future::Future;

use super::Http2Server;
use super::Http2ServerOptions;
use crate::Request;
use crate::Response;

pub fn create_server<F, Fut>(
  options: Http2ServerOptions,
  handle_func: F,
) -> Http2Server<F, Fut>
where
  F: 'static + Send + Sync + Fn(Request, Response) -> Fut,
  Fut: 'static + Send + Future<Output = crate::Result<()>>,
{
  Http2Server::new(handle_func, options)
}
