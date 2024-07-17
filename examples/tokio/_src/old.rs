// mod stream_body;

use std::convert::Infallible;
use std::future::Future;
use std::io;
use std::net::SocketAddr;
use std::pin::Pin;
use std::sync::Arc;
use std::time::Duration;

use futures::AsyncReadExt;
use futures::TryStreamExt;
use http_body_util::combinators::BoxBody;
use http_body_util::BodyExt;
use http_body_util::BodyStream;
use http_body_util::Full;
use http_body_util::StreamBody;
use hyper::body::Bytes;
use hyper::header::HeaderValue;
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::service::Service;
use hyper_util::rt::TokioIo;
use tokio::io::AsyncWriteExt;
use tokio::io::DuplexStream;
use tokio::net::TcpListener;
use tokio_util::codec::BytesCodec;
use tokio_util::codec::FramedRead;
use uhttp::body_parser;
use uhttp::Request;

// use self::stream_body::StreamBody;

#[tokio::main]
async fn main() {
  start(|mut req| async move {
    println!("URL {}", req.url);

    let b = body_parser::utf8(&mut req.body).await?;
    println!("BODY {}", b);

    Ok(())
  })
  .await
  .unwrap();
}

async fn start<F, Fut>(handler: F) -> Result<(), Box<dyn std::error::Error + Send + Sync>>
where
  F: Send + Sync + 'static + Fn(Request) -> Fut,
  Fut: Future<Output = io::Result<()>> + Send + Sync + 'static,
{
  let handler = Arc::new(handler);

  let addr = SocketAddr::from(([127, 0, 0, 1], 8080));

  let listener = TcpListener::bind(addr).await?;

  loop {
    let (stream, _) = listener.accept().await?;
    let io = TokioIo::new(stream);

    let handler = handler.clone();

    tokio::task::spawn(async move {
      if let Err(err) = http1::Builder::new()
        .serve_connection(
          io,
          service_fn(
            move |req: hyper::Request<hyper::body::Incoming>| -> Pin<
              Box<
                dyn Send
                  + Sync
                  + Future<
                    Output = Result<hyper::Response<BoxBody<Bytes, std::io::Error>>, Infallible>,
                  >,
              >,
            > {
              let handler = handler.clone();

              let (mut asyncwriter, asyncreader) = tokio::io::duplex(1024);
              let reader_stream = tokio_util::io::ReaderStream::new(asyncreader);
              let stream_body = StreamBody::new(reader_stream.map_ok(hyper::body::Frame::data));
              let boxed_body = stream_body.boxed();
              let response = hyper::Response::builder()
                .header("Content-Type", "text/html")
                .status(hyper::StatusCode::OK)
                .body(boxed_body)
                .unwrap();

              tokio::task::spawn(async move {
                  asyncwriter.write(b"hello world").await.unwrap();
              });

              Box::pin(async move {
                // let (mut writer, body) = StreamBody::channel();

                Ok(response)

                // Ok(Response::builder().body(body).unwrap())
                // Ok(hyper::Response::new(Full::new(Bytes::from("Hello, World!"))))
              })
            },
          ),
        )
        .await
      {
        eprintln!("Error serving connection: {:?}", err);
      }
  });
  }
}

// #[derive(Debug, Clone)]
// struct Svc {
// }

// impl Service<hyper::Request<hyper::body::Incoming>> for Svc {
//     type Response = hyper::Response<Full<Bytes>>;
//     type Error = hyper::Error;
//     type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

//     fn call(&self, req: hyper::Request<hyper::body::Incoming>) -> Self::Future {
//       let res = hyper::Response::builder().;

//       Box::pin(async { res })
//     }
// }

/*

                dyn Send + Sync + Future<Output = Result<hyper::Response<Full<Bytes>>, Infallible>>,


tokio::task::spawn(async move {
                  let method = req.method().as_str().to_string();
                  let url = req.uri().to_string();
                  let host = req
                    .headers()
                    .get(hyper::header::HOST)
                    .unwrap_or(&HeaderValue::from_static(""))
                    .to_str()
                    .unwrap()
                    .to_string();

                  let body = req.into_body();
                  let stream = body.into_data_stream();
                  let reader = stream
                    .map_err(|error| io::Error::other(error))
                    .into_async_read();

                  let request = uhttp::Request {
                    method,
                    url,
                    proto: String::new(), //req.version(),
                    headers: uhttp::Headers::default(),
                    body: Box::new(reader),
                    host,
                  };

                  // req.

                  // let response = uhttp::HttpResponse {
                  //   headers: uhttp::Headers::default(),
                  //   stream: Box::new(),
                  // };

                  handler(request).await.unwrap();
                });


*/
