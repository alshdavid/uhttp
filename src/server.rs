use std::io;
use std::io::Read;
use std::net::TcpListener;
use std::net::ToSocketAddrs;

use crate::Headers;
use crate::HttpResponse;

use super::Request;
use super::Response;

static RC: u8 = b'\r';
static NL: u8 = b'\n';

pub struct Server {
  handler: Box<dyn Fn(Request, Box<dyn Response>) -> io::Result<()>>,
}

impl Server {
  pub fn new(handler: impl Fn(Request, Box<dyn Response>) -> io::Result<()> + 'static) -> Self {
    Self {
      handler: Box::new(handler),
    }
  }

  pub fn listen<A: ToSocketAddrs>(
    &self,
    addr: A,
  ) -> io::Result<()> {
    let listener = TcpListener::bind(addr)?;

    while let Ok((mut stream, _)) = listener.accept() {      
      let mut header_bytes = Vec::<u8>::new();
      let mut header_count = 0;

      let mut buf = [0u8; 1];
      let mut rc1 = false;
      let mut nl1 = false;
      let mut rc2 = false;
      let mut nl2 = false;

      loop {
        stream.read_exact(&mut buf)?;
        let v = buf[0];

        if rc1 == true && nl1 == true && rc2 == true && nl2 == true {
          break;
        } else if rc1 == false && v == RC {
          rc1 = true;
        } else if rc1 == true && nl1 == false && v == NL {
          nl1 = true;
          header_count += 1;
        } else if rc1 == true && nl1 == true && rc2 == false && v == RC {
          rc2 = true;
        } else if rc1 == true && nl1 == true && rc2 == true && nl2 == false && v == NL {
          nl2 = true;
        } else {
          rc1 = false;
          nl1 = false;
          rc2 = false;
          nl2 = false;
        }

        header_bytes.push(buf[0]);
      }

      let mut raw_headers = vec![httparse::EMPTY_HEADER; header_count - 1];

      let mut raw_headers_parser = httparse::Request::new(&mut raw_headers);
      raw_headers_parser.parse(&header_bytes).map_err(|e| io::Error::other(e))?;

      let req_version = raw_headers_parser.version.unwrap_or(0);

      let mut request = Request {
        method: raw_headers_parser.method.unwrap_or("GET").to_string(),
        url: raw_headers_parser.path.unwrap_or("/").to_string(),
        proto: format!("HTTP/1.{}", req_version),
        proto_major: 1,
        proto_minor: req_version,
        headers: Headers::default(),
        body: Box::new(stream.try_clone()?),
        host: Default::default(),
      };

      drop(raw_headers_parser);

      for header in raw_headers.drain(0..) {
        request.headers.internal.insert(header.name.to_string(), unsafe { String::from_utf8_unchecked(header.value.to_owned()) });
      }

      if let Some(host) = request.headers.get("host") {
        request.host = host.clone();
      }

      let response = HttpResponse {
        headers: Default::default(),
        stream,
      };

      (*self.handler)(request, Box::new(response))?;
    }

    Ok(())
  }
}
