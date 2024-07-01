use std::io;

use futures::Future;
use tokio::io::AsyncReadExt;
use tokio::net::TcpListener;
use tokio::net::ToSocketAddrs;

use super::HttpRequestReader;
use super::HttpResponse;
use super::Request;
use super::ResponseRef;
use crate::constants::DEFAULT_METHOD;
use crate::constants::DEFAULT_URL;
use crate::constants::HEADER_CONTENT_LENGTH;
use crate::constants::NL;
use crate::constants::RC;
use crate::Headers;

pub struct Server<Fut: Future<Output = io::Result<()>>> {
  handler: Box<dyn Fn(Request, ResponseRef) -> Fut>,
}

impl<Fut: Future<Output = io::Result<()>>> Server<Fut> {
  pub fn new(handler: impl Fn(Request, ResponseRef) -> Fut + 'static) -> Self {
    Self {
      handler: Box::new(handler),
    }
  }

  pub async fn listen<A: ToSocketAddrs>(
    &self,
    addr: A,
  ) -> io::Result<()> {
    let listener = TcpListener::bind(addr).await?;

    while let Ok((mut stream, _)) = listener.accept().await {
      println!("connect");
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
      let mut content_length: usize = 0;

      for header in raw_headers.drain(0..) {
        let key = header.name.to_string().to_lowercase();

        let values = unsafe { String::from_utf8_unchecked(header.value.to_owned()) };

        if key == HEADER_CONTENT_LENGTH {
          content_length = values.parse::<usize>().unwrap();
        }

        headers.set(key, values)
      }

      let (reader, writer) = stream.into_split();

      let request = Request {
        method,
        url,
        proto: format!("HTTP/1.{}", req_version),
        headers,
        host,
        body: Box::new(HttpRequestReader {
          content_length,
          cursor: 0,
          stream: reader,
        }),
      };

      let response = HttpResponse {
        headers: Default::default(),
        stream: writer,
      };

      (*self.handler)(request, Box::new(response)).await?;
    }

    Ok(())
  }
}
