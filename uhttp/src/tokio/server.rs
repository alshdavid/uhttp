use std::future::Future;
use std::io;
use std::sync::Arc;

use tokio::io::AsyncReadExt;
use tokio::net::TcpListener;
use tokio::net::ToSocketAddrs;

use super::HttpResponse;
use super::Request;
use super::ResponseRef;
use crate::constants::DEFAULT_METHOD;
use crate::constants::DEFAULT_URL;
use crate::constants::NL;
use crate::constants::RC;
use crate::Headers;

pub struct Server<F, Fut>
where
  F: Fn(Request, ResponseRef) -> Fut + Send + Sync,
  Fut: Future<Output = io::Result<()>> + Send + Sync + 'static,
{
  handler: Arc<F>,
}

impl<F, Fut> Server<F, Fut>
where
  F: Fn(Request, ResponseRef) -> Fut + 'static + Send + Sync,
  Fut: Future<Output = io::Result<()>> + Send + Sync + 'static,
{
  pub fn new(handler: F) -> Self {
    Self {
      handler: Arc::new(handler),
    }
  }

  pub async fn listen<A: ToSocketAddrs>(
    &self,
    addr: A,
  ) -> io::Result<()> {
    let listener = TcpListener::bind(addr).await?;

    while let Ok((mut stream, _)) = listener.accept().await {
      let handler = self.handler.clone();
      tokio::spawn(async move {
        let mut header_bytes = Vec::<u8>::new();
        let mut header_count = 0;

        let mut buf = [0u8; 1];

        let mut rc1 = false;
        let mut nl1 = false;
        let mut rc2 = false;

        loop {
          stream.read(&mut buf).await?;
          let v = buf[0];

          if rc1 == false && v == RC {
            rc1 = true;
          } else if rc1 == true && nl1 == false && v == NL {
            nl1 = true;
            header_count += 1;
          } else if rc1 == true && nl1 == true && rc2 == false && v == RC {
            rc2 = true;
          } else if rc1 == true && nl1 == true && rc2 == true && v == NL {
            break;
          } else {
            rc1 = false;
            nl1 = false;
            rc2 = false;
          }

          header_bytes.push(buf[0]);
        }

        let mut raw_headers = vec![httparse::EMPTY_HEADER; header_count - 1];
        let mut raw_headers_parser = httparse::Request::new(&mut raw_headers);

        raw_headers_parser
          .parse(&header_bytes)
          .map_err(|e| io::Error::other(e))?;

        let req_version = raw_headers_parser.version.unwrap_or(0);
        let method = raw_headers_parser
          .method
          .unwrap_or(DEFAULT_METHOD)
          .to_string();
        let url = raw_headers_parser.path.unwrap_or(DEFAULT_URL).to_string();
        let host = unsafe { String::from_utf8_unchecked(raw_headers[0].value.to_owned()) };

        let mut headers = Headers::default();

        for header in raw_headers.drain(0..) {
          let key = header.name.to_string().to_lowercase();
          let values = unsafe { String::from_utf8_unchecked(header.value.to_owned()) };
          headers.set(key, values)
        }

        let (reader, writer) = stream.into_split();

        let request = Request {
          method,
          url,
          proto: format!("HTTP/1.{}", req_version),
          headers,
          host,
          body: Box::new(reader),
        };

        let response = HttpResponse {
          headers: Default::default(),
          stream: Box::new(writer),
        };

        handler(request, Box::new(response)).await
      });
    }

    Ok(())
  }
}
