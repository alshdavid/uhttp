use http::HeaderMap;
use http_body_util::Full;
use http_body_util::combinators::BoxBody;
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper_util::rt::TokioIo;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpListener;
use tokio::net::ToSocketAddrs;
use tokio::sync::mpsc::unbounded_channel;

use crate::HandlerFunc;

use super::internal_types::HyperBytes;
use super::internal_types::HyperResponse;
use super::create_stream::create_stream;
use super::response::Http1Response;

/// Simple wrapper around hyper to make it a little nicer to use
pub async fn create_server(
  addr: impl ToSocketAddrs,
  handle_func: impl HandlerFunc,
) -> anyhow::Result<()> {
  let listener = TcpListener::bind(&addr).await?;

  loop {
    let Ok((stream, _)) = listener.accept().await else {
      continue;
    };
    let io = TokioIo::new(stream);

    tokio::task::spawn({
      async move {
        http1::Builder::new()
          .serve_connection(
            io,
            service_fn({
              move |req| {
                async move {
                  let (tx_writer, mut rx_writer) = unbounded_channel();


                  let mut res = Http1Response {
                    tx_writer,
                    headers: HeaderMap::new(),
                  };

                  handle_func(req, &mut res).await.unwrap();

                  drop(res.tx_writer);
                  let mut res_headers = res.headers;

                  // Drain the channel
                  let mut buf = vec![];
                  while let Ok(bytes) = rx_writer.try_recv() {
                    buf.extend(bytes);
                  }

                  // If the request specifies that it's an event stream or chunked transfer encoding, return a stream writer
                  let content_type_event_stream = res_headers
                    .get("Content-Type")
                    .is_some_and(|v| v == "text/event-stream");
                  
                  let content_transfer_encoding_chunked = res_headers
                    .get("Content-Transfer-Encoding")
                    .is_some_and(|v| v == "chunked");

                  if content_type_event_stream || content_transfer_encoding_chunked {
                    let mut hyper_res = HyperResponse::builder();
                    std::mem::swap(&mut res_headers, hyper_res.headers_mut().unwrap());

                    let (res, mut writer) = create_stream(hyper_res).unwrap();
                    tokio::task::spawn(async move {
                      if let Some(bytes) = rx_writer.recv().await {
                        writer.write(&bytes).await.unwrap();
                      }
                    });

                    return Ok(res);
                  }

                  // If the channel is still open, send back a chunked response
                  if let Some(bytes) = rx_writer.recv().await {
                    let mut hyper_res = HyperResponse::builder();
                    std::mem::swap(&mut res_headers, hyper_res.headers_mut().unwrap());
                    let hyper_res = hyper_res
                      .header("Content-Transfer-Encoding", "chunked")
                      .header("X-Content-Type-Options", "nosniff");

                    let (hyper_res, mut writer) = create_stream(hyper_res).unwrap();

                    tokio::task::spawn(async move {
                      buf.extend(bytes);
                      writer.write(&buf).await.unwrap();
                      writer.flush().await.unwrap();

                      while let Some(bytes) = rx_writer.recv().await {
                        writer.write(&bytes).await.unwrap();
                        writer.flush().await.unwrap();
                      }

                      // writer.write(b"\r\n").await.unwrap();
                      writer.flush().await.unwrap();
                    });

                    Ok(hyper_res)
                  }
                  // If the channel is closed, emit everything to client
                  else {
                    let mut hyper_res = HyperResponse::builder();
                    std::mem::swap(&mut res_headers, hyper_res.headers_mut().unwrap());
                    let b = BoxBody::new(Full::new(HyperBytes::from(buf)));
                    hyper_res.body(b)
                  }
                }
              }
            }),
          )
          .await
          .ok();
      }
    });
  }
}

