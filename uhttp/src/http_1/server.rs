use std::collections::HashMap;
use std::io;
use std::io::Read;
use std::io::Write;
use std::net::ToSocketAddrs;
use std::sync::Arc;

use may;
use may::go;
use may::net::TcpListener;
use may::sync::mpmc;
use may::sync::mpsc;
use may::sync::spsc;

use super::request_reader::RequestReader;
use super::response_writer::ResponseWriteAction;
use super::response_writer::ResponseWriter;
use crate::c;
use crate::Headers;
use crate::Request;
use crate::Response;

pub struct Server<Handler>
where
  Handler: Fn(Request, Response) -> io::Result<()> + 'static + Sync + Send,
{
  handler: Arc<Handler>,
}

impl<Handler> Server<Handler>
where
  Handler: Fn(Request, Response) -> io::Result<()> + 'static + Sync + Send,
{
  pub fn new(handler: Handler) -> Self {
    Self {
      handler: Arc::new(handler),
    }
  }

  pub fn listen<A: ToSocketAddrs>(
    &self,
    addr: A,
  ) -> io::Result<()> {
    may::config().set_workers(8);

    let (tx_init, rx_init) = mpmc::channel::<(Request, Response)>();

    // Spawn worker threads. In 2024, OS threads are cheap to spawn
    // so delegating HTTP request handling to OS threads is fast and
    // allows the developer to avoid using async Rust.
    // TODO: auto-grow if connections exceed the pool size
    for _ in 0..1000 {
      let rx_init = rx_init.clone();
      let handler = self.handler.clone();

      // Run each request in its own dedicated thread
      std::thread::spawn(move || {
        while let Ok((request, response)) = rx_init.recv() {
          handler(request, response).unwrap();
        }
      });
    }

    let listener = TcpListener::bind(addr)?;

    while let Ok((mut stream, _)) = listener.accept() {
      let tx_init = tx_init.clone();
      let (tx_write, rx_write) = mpsc::channel::<ResponseWriteAction>();

      go!({
        let mut stream = stream.try_clone().unwrap();
        move || {
          while let Ok(event) = rx_write.recv() {
            match event {
              ResponseWriteAction::Write(bytes) => {
                stream.write_all(&bytes).unwrap();
              }
              ResponseWriteAction::WriteAll(bytes) => {
                stream.write_all(&bytes).unwrap();
              }
              ResponseWriteAction::Flush => {
                stream.flush().unwrap();
              }
            }
          }
        }
      });

      go!(move || {
        let mut buf = Vec::<u8>::new();

        let mut header_count = 0;
        let mut body_start = 0;
        let mut cursor = 0;
        let mut rc0 = false;
        let mut nl0 = false;
        let mut rc1 = false;

        // NOTE: Socket connection can be reused, incoming bytes
        // from multiple requests will be added in order
        'socket: loop {
          'block: {
            // Look for message body from incoming stream
            // TODO: A more reliable way to do this
            let pos = cursor;
            cursor = buf.len();

            for i in pos..buf.len() {
              if rc0 == false && buf[i] == c::chars::RC {
                rc0 = true;
              } else if rc0 == true && nl0 == false && buf[i] == c::chars::NL {
                nl0 = true;
                header_count += 1;
              } else if rc0 == true && nl0 == true && rc1 == false && buf[i] == c::chars::RC {
                rc1 = true;
              } else if rc0 == true && nl0 == true && rc1 == true && buf[i] == c::chars::NL {
                body_start = i + 1;
                break 'block;
              } else {
                rc0 = false;
                nl0 = false;
                rc1 = false;
              }
            }

            // If we are not in a request, wait for more bytes
            let mut temp_buf = vec![0; c::buffer::DEFAULT];
            match stream.read(&mut temp_buf) {
              Ok(n) => {
                if n == 0 {
                  break 'socket;
                }
                buf.extend(temp_buf.drain(0..n));
              }
              Err(err) => println!("err = {err:?}"),
            }
          };

          // If we are not in a request
          if body_start == 0 {
            continue;
          }

          // Parse headers
          let header_bytes = buf.drain(0..body_start).collect::<Vec<u8>>();
          let mut raw_headers = vec![httparse::EMPTY_HEADER; header_count - 1];
          let mut raw_request = httparse::Request::new(&mut raw_headers);
          raw_request.parse(&header_bytes).unwrap();

          let mut headers = HashMap::<String, Vec<String>>::new();
          for i in 0..raw_request.headers.len() {
            let mut header = std::mem::replace(&mut raw_request.headers[i], httparse::EMPTY_HEADER);
            let header_name = header.name.to_string();
            let header_value = std::mem::take(&mut header.value).to_owned();
            let header_value = unsafe { String::from_utf8_unchecked(header_value) };
            headers
              .entry(header_name.to_lowercase())
              .or_default()
              .push(header_value);
          }

          // Determine content length
          let mut content_length = 0;
          if let Some(v) = headers.get(c::headers::CONTENT_LENGTH) {
            content_length = v.get(0).unwrap().parse::<usize>().unwrap();
          }

          // Forward reader/writer to dedicated thread
          let (tx_read, rx_read) = spsc::channel::<Vec<u8>>();

          let req = Request {
            method: raw_request.method.unwrap().to_string(),
            url: raw_request.path.unwrap().to_string(),
            proto: format!("HTTP/1.{}", raw_request.version.unwrap()),
            headers: Headers::from(headers),
            body: Box::new(RequestReader::new(rx_read)),
            host: Default::default(),
          };

          let res = Response::new(ResponseWriter::new(tx_write.clone()));

          tx_init.send((req, res)).unwrap();

          // Skip if the request does not have a body
          if content_length == 0 {
            drop(tx_read);
            header_count = 0;
            body_start = 0;
            continue;
          }

          // Read bytes for body until Content-Length
          // TODO Transfer-Encoding: chunked
          let mut bytes_read = 0;

          loop {
            let mut bytes_to_take = content_length - bytes_read;
            if bytes_to_take > buf.len() {
              bytes_to_take = buf.len();
            }

            let bytes = buf.drain(0..bytes_to_take).collect::<Vec<u8>>();

            if bytes.len() > 0 {
              bytes_read += bytes.len();
              tx_read.send(bytes).unwrap();
            }

            if bytes_read == content_length {
              drop(tx_read);
              header_count = 0;
              body_start = 0;
              break;
            }

            let mut temp_buf = vec![0; c::buffer::DEFAULT];
            match stream.read(&mut temp_buf) {
              Ok(n) => {
                if n == 0 {
                  break;
                }

                buf.extend(temp_buf.drain(0..n));
              }
              Err(err) => println!("err = {err:?}"),
            }
          }
        }
      });
    }

    Ok(())
  }
}
