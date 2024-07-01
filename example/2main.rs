use std::collections::HashMap;
use std::fmt::Debug;
use std::io::Read;
use std::io::Write;
use std::io::{self};
use std::net::SocketAddr;
use std::net::TcpListener;
use std::pin::Pin;
use std::thread;
use std::time::Duration;

use uhttp::Server;

static RC: u8 = b'\r';
static NL: u8 = b'\n';

fn main() -> io::Result<()> {
  let listener = TcpListener::bind("0.0.0.0:3000")?;
  let ip = listener.local_addr()?.ip();
  let port = listener.local_addr()?.port();

  println!("Listening on http://{}:{}", ip, port);

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
      } else if rc1 == false && v == RC {
        rc1 = true;
      } else if rc1 == true && nl1 == false && v == NL {
        nl1 = true;
        header_count += 1;
      } else if rc1 == true && nl1 == true && rc2 == false && v == RC {
        rc2 = true;
      } else if rc1 == true && nl1 == true && rc2 == true && nl2 == false && v == NL {
        nl2 = true;
        break;

      } else {
        rc1 = false;
        nl1 = false;
        rc2 = false;
        nl2 = false;
      }

      header_bytes.push(buf[0]);
    }

    let mut buf = [0u8; 1];

    loop {
      stream.read_exact(&mut buf)?;
      let v = buf[0];
      println!("{}", String::from_utf8(vec![v]).unwrap());
    }


    return Ok(());

    let mut request_headers = vec![httparse::EMPTY_HEADER; header_count - 1];
    let mut req = httparse::Request::new(&mut request_headers);
    req.parse(&header_bytes).map_err(|e| io::Error::other(e))?;



    let mut response_headers = HashMap::<String, Vec<String>>::new();
    response_headers.insert(
      "Access-Control-Allow-Origin".to_string(),
      vec!["*".to_string()],
    );
    response_headers.insert(
      "Access-Control-Allow-Methods".to_string(),
      vec!["OPTIONS".to_string(), "GET".to_string(), "POST".to_string()],
    );

    let response_code = format!("{}", 201);
    let response_message = "Whatever";

    // Write Reply
    stream.write_all(b"HTTP/1.1 ")?;
    // Status Code
    stream.write_all(response_code.as_bytes())?;
    stream.write_all(b" ")?;

    // Message
    stream.write_all(response_message.as_bytes())?;
    stream.write_all(b"\r\n")?;

    // Write headers
    for (key, values) in response_headers.iter() {
      stream.write_all(key.as_bytes())?;
      stream.write_all(b": ")?;

      for (i, value) in values.iter().enumerate() {
        stream.write_all(value.as_bytes())?;
        if i != values.len() - 1 {
          stream.write_all(b", ")?;
        }
      }

      stream.write_all(b"\r\n")?;
    }

    // Start body
    stream.write_all(b"\r\n")?;
  }

  Ok(())
}



use std::collections::VecDeque;
use std::io;
use std::io::Read;
use std::net::TcpListener;
use std::net::TcpStream;
use std::net::ToSocketAddrs;
use std::sync::mpsc::channel;
use std::sync::mpsc::Sender;
use std::sync::Arc;
use std::sync::Mutex;
use std::thread;
use std::thread::JoinHandle;

use super::HttpRequestReader;
use super::HttpResponse;
use super::Request;
use super::Response;
use crate::constants::DEFAULT_METHOD;
use crate::constants::DEFAULT_URL;
use crate::constants::HEADER_CONTENT_LENGTH;
use crate::constants::NL;
use crate::constants::RC;
use crate::Headers;

pub struct Server {
  handler: Arc<dyn Fn(Request, Box<dyn Response>) -> io::Result<()> + Send + Sync>,
}

impl Server {
  pub fn new(handler: impl Fn(Request, Box<dyn Response>) -> io::Result<()> + Send + Sync + 'static) -> Self {
    Self {
      handler: Arc::new(handler),
    }
  }

  pub fn listen<A: ToSocketAddrs>(
    &self,
    addr: A,
  ) -> io::Result<()> {
    let listener = TcpListener::bind(addr)?;
    let worker_count = 1;

    let mut senders = vec![];

    for tid in 0..worker_count {
      let (tx_queue, rx_queue) = channel::<TcpStream>();
      senders.push(tx_queue);

      let handle: JoinHandle<io::Result<()>> = thread::spawn({
        let handler = self.handler.clone();
        move || {
          while let Ok(mut stream) = rx_queue.recv() {
            println!("{} HIT", tid);
            let mut header_bytes = Vec::<u8>::new();
            let mut header_count = 0;
      
            let mut buf = [0u8; 1];
      
            let mut rc1 = false;
            let mut nl1 = false;
            let mut rc2 = false;
      
            loop {
              stream.read(&mut buf)?;
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
      
            let request = Request {
              method,
              url,
              proto: format!("HTTP/1.{}", req_version),
              headers,
              host,
              body: Box::new(HttpRequestReader {
                content_length,
                cursor: 0,
                stream: stream.try_clone()?,
              }),
            };
      
            let response = HttpResponse {
              headers: Default::default(),
              stream,
            };
      
            (*handler)(request, Box::new(response))?;
          }
  
          Ok(())
        }
      });
    }

    let mut current_worker = 0;

    while let Ok((stream, _)) = listener.accept() {
      println!("{}", current_worker);
      senders[current_worker].send(stream).unwrap();
      if current_worker == worker_count - 1 {
        current_worker = 0;
      } else {
        current_worker += 1;
      }
    }

    Ok(())
  }
}
