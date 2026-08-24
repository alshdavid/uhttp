use std::convert::Infallible;
use std::future::Future;
use std::sync::Arc;

use http_body_util::Full;
use http_body_util::combinators::BoxBody;
use hyper::Response as HyperResponse;
use hyper::body::Bytes as HyperBytes;
use hyper::service::service_fn;
use hyper_util::rt::TokioExecutor;
use hyper_util::rt::TokioIo;
use hyper_util::server::conn::auto;
use tokio::net::TcpListener;
use tokio::net::ToSocketAddrs;

use crate::Request;
use crate::Response;

pub struct Http1Server<F, Fut>
where
  F: 'static + Send + Sync + Fn(Request, Response) -> Fut,
  Fut: 'static + Send + Future<Output = crate::Result<()>>,
{
  handle_func: Arc<F>,
}

impl<F, Fut> Http1Server<F, Fut>
where
  F: 'static + Send + Sync + Fn(Request, Response) -> Fut,
  Fut: 'static + Send + Future<Output = crate::Result<()>>,
{
  pub fn new(handle_func: F) -> Self {
    Self {
      handle_func: Arc::new(handle_func),
    }
  }

  pub async fn listen(
    &self,
    addr: impl ToSocketAddrs,
  ) -> crate::Result<()> {
    let listener: TcpListener = TcpListener::bind(&addr).await?;
    let handler_func_ref = Arc::clone(&self.handle_func);

    loop {
      let Ok((stream, _)) = listener.accept().await else {
        continue;
      };

      let handler_func_ref = handler_func_ref.clone();

      tokio::task::spawn(async move {
        let io = TokioIo::new(stream);

        let service_builder = auto::Builder::new(TokioExecutor::new());
        let service_handler = service_fn(move |req| {
          let request = Request::new(req);

          let (tx_res, rx_res) = tokio::sync::oneshot::channel();
          let response = Response::new(tx_res, HyperResponse::builder());

          let fut = handler_func_ref(request, response);
          let (tx_fut_res, mut rx_fut_res) = tokio::sync::oneshot::channel::<crate::Result<()>>();

          tokio::task::spawn(async move {
            match fut.await {
              Ok(_handler_response) => {}
              Err(handler_error) => drop(tx_fut_res.send(Err(handler_error))),
            };
          });

          async move {
            Ok::<HyperResponse<BoxBody<HyperBytes, Infallible>>, crate::Error>(match rx_res.await {
              Ok(res) => {
                if let Ok(Err(err)) = rx_fut_res.try_recv() {
                  return Ok(handle_error(crate::Error::generic(format!("{}", err))));
                };
                res
              }
              Err(err) => handle_error(crate::Error::generic(format!("{}", err))),
            })
          }
        });

        #[cfg(feature = "websocket")]
        service_builder
          .serve_connection_with_upgrades(io, service_handler)
          .await
          .ok();

        #[cfg(not(feature = "websocket"))]
        service_builder
          .serve_connection(io, service_handler)
          .await
          .ok();
      });
    }
  }
}

fn handle_error(error: impl std::fmt::Display) -> HyperResponse<BoxBody<HyperBytes, Infallible>> {
  let content = HyperBytes::from(format!("{}", error));
  let body = BoxBody::new(Full::new(content));
  let response = HyperResponse::builder().status(500).body(body);
  let Ok(response) = response else { todo!() };
  response
}
