use std::convert::Infallible;
use std::future::Future;
use std::sync::Arc;

use http_body_util::Full;
use http_body_util::combinators::BoxBody;
use hyper::Response as HyperResponse;
use hyper::body::Bytes as HyperBytes;
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper_util::rt::TokioIo;
use tokio::net::TcpListener;
use tokio::net::ToSocketAddrs;

use super::Request;
use super::Response;

pub async fn http1_server<F, Fut, A>(
  addr: A,
  handle_func: F,
) -> anyhow::Result<()>
where
  A: ToSocketAddrs,
  F: 'static + Send + Sync + Fn(Request, Response) -> Fut,
  Fut: 'static + Send + Future<Output = anyhow::Result<()>>,
{
  let listener = TcpListener::bind(&addr).await?;
  let handler_func_ref = Arc::new(handle_func);

  loop {
    let Ok((stream, _)) = listener.accept().await else {
      continue;
    };

    let io = TokioIo::new(stream);
    let handler_func_ref = handler_func_ref.clone();

    tokio::task::spawn(async move {
      let service_builder = http1::Builder::new();
      let service_handler = service_fn(move |req| {
        let request = Request::new(req);

        let (tx_res, rx_res) = tokio::sync::oneshot::channel();
        let response = Response::new(tx_res, HyperResponse::builder());

        let fut = handler_func_ref(request, response);

        tokio::task::spawn(async move {
          match fut.await {
            Ok(handler_response) => handler_response,
            Err(handler_error) => panic!("Unable to complete request {}", handler_error),
          };
        });

        async move {
          Ok::<HyperResponse<BoxBody<HyperBytes, Infallible>>, anyhow::Error>(match rx_res.await {
            Ok(res) => res,
            Err(err) => handle_error(anyhow::anyhow!("Unable to complete request {}", err)),
          })
        }
      });

      service_builder
        .serve_connection(io, service_handler)
        .await
        .ok();
    });
  }
}

fn handle_error(error: anyhow::Error) -> HyperResponse<BoxBody<HyperBytes, Infallible>> {
  let content = HyperBytes::from(format!("{}", error));
  let body = BoxBody::new(Full::new(content));
  let response = HyperResponse::builder().status(500).body(body);
  let Ok(response) = response else { todo!() };
  response
}
