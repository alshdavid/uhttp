use std::convert::Infallible;
use std::future::Future;
use std::path::PathBuf;
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
use tokio_rustls::TlsAcceptor;

use super::tls::load_certs_and_key;
use crate::Request;
use crate::Response;

#[derive(Debug, Clone, Default)]
pub struct Http2ServerOptions {
  pub cert_path: Option<PathBuf>,
  pub key_path: Option<PathBuf>,
}

pub struct Http2Server<F, Fut>
where
  F: 'static + Send + Sync + Fn(Request, Response) -> Fut,
  Fut: 'static + Send + Future<Output = crate::Result<()>>,
{
  handle_func: Arc<F>,
  options: Arc<Http2ServerOptions>,
}

impl<F, Fut> Http2Server<F, Fut>
where
  F: 'static + Send + Sync + Fn(Request, Response) -> Fut,
  Fut: 'static + Send + Future<Output = crate::Result<()>>,
{
  pub fn new(
    handle_func: F,
    options: Http2ServerOptions,
  ) -> Self {
    Self {
      handle_func: Arc::new(handle_func),
      options: Arc::new(options),
    }
  }

  pub async fn listen(
    &self,
    addr: impl ToSocketAddrs,
  ) -> crate::Result<()> {
    let listener: TcpListener = TcpListener::bind(&addr).await?;
    let handler_func_ref = Arc::clone(&self.handle_func);

    let Some(cert_path) = self.options.cert_path.as_ref() else {
      return Err(crate::Error::generic(
        "TLS certificate path is required for HTTP/2 server",
      ));
    };

    let Some(key_path) = self.options.key_path.as_ref() else {
      return Err(crate::Error::generic(
        "TLS key path is required for HTTP/2 server",
      ));
    };

    let tls_config = load_certs_and_key(cert_path, key_path)?;
    let acceptor = TlsAcceptor::from(tls_config);

    loop {
      let Ok((stream, _)) = listener.accept().await else {
        continue;
      };

      let acceptor = acceptor.clone();
      let handler_func_ref = handler_func_ref.clone();

      tokio::task::spawn(async move {
        let tls_stream = match acceptor.accept(stream).await {
          Ok(s) => s,
          Err(e) => {
            eprintln!("TLS handshake error: {}", e);
            return;
          }
        };

        let io = TokioIo::new(tls_stream);

        let service_builder = auto::Builder::new(TokioExecutor::new());
        let service_handler = service_fn(move |req| {
          let request = Request::new(req);

          let (tx_res, rx_res) = tokio::sync::oneshot::channel();
          let response = Response::new(tx_res, HyperResponse::builder());

          let fut = handler_func_ref(request, response);

          tokio::task::spawn(async move {
            match fut.await {
              Ok(_handler_response) => {}
              Err(_handler_error) => {}
            };
          });

          async move {
            Ok::<HyperResponse<BoxBody<HyperBytes, Infallible>>, crate::Error>(match rx_res.await {
              Ok(res) => res,
              Err(err) => handle_error(crate::Error::generic(format!(
                "Unable to complete request {}",
                err
              ))),
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

fn handle_error(error: crate::Error) -> HyperResponse<BoxBody<HyperBytes, Infallible>> {
  let content = HyperBytes::from(format!("{}", error));
  let body = BoxBody::new(Full::new(content));
  let response = HyperResponse::builder().status(500).body(body);
  let Ok(response) = response else { todo!() };
  response
}
