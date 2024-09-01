use std::cell::RefCell;
use std::collections::HashMap;
use std::future::Future;
use std::io;
use std::marker::PhantomData;
use std::net::SocketAddr;
use std::net::ToSocketAddrs;
use std::time::Duration;

use bytes::BufMut;
use bytes::BytesMut;
use socket2::Domain;
use socket2::Socket;
use socket2::Type;
use tokio;
use tokio::io::AsyncReadExt;
use tokio::io::AsyncWriteExt;

use crate::c;
use crate::Headers;
use crate::Request;
use crate::Response;

pub struct Server<Handler, Fut>
where
  Handler: Fn(Request, Response) -> Fut + Send + Copy + 'static,
  Fut: Future<Output = io::Result<()>> + Send,
{
  handler: RefCell<Option<Handler>>,
  p0: PhantomData<Fut>,
}

impl<Handler, Fut> Server<Handler, Fut>
where
  Handler: Fn(Request, Response) -> Fut + Send + Copy + 'static,
  Fut: Future<Output = io::Result<()>> + Send,
{
  pub fn new(handler: Handler) -> Self {
    Self {
      handler: RefCell::new(Some(handler)),
      p0: PhantomData::default(),
    }
  }

  pub async fn listen<A: ToSocketAddrs>(
    &self,
    addr: A,
  ) -> io::Result<()> {
    let listener = create_listener(addr)?;
    let handler = self.handler.borrow_mut().take().unwrap();
    
    while let Ok((stream, _)) = listener.accept().await {
      let (mut reader, mut writer) = stream.into_split();
      
      // Task dedicated to writing to the socket
      let (tx_writer, mut rx_writer) = tokio::sync::mpsc::unbounded_channel::<Vec<u8>>();
      tokio::task::spawn(async move {
        while let Some(bytes) = rx_writer.recv().await {
          writer.write_all(&bytes).await.unwrap();
        }
      });
      
      // Task dedicated to reading from the socket
      tokio::task::spawn(async move {
        let mut buf_temp = Box::new([0; c::buffer::DEFAULT]);
        let mut buf = BytesMut::new();
  
        // Read from the socket
        'socket: loop {
          let mut header_count = 0;
          let mut body_start = 0;
          let mut cursor = 0;
          let mut rc0 = false;
          let mut nl0 = false;
          let mut rc1 = false;

          // Detect incoming headers
          'get_headers: loop {          
            // Look for headers and detect start of body
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
                break 'get_headers;
              } else {
                rc0 = false;
                nl0 = false;
                rc1 = false;
              }
            }
            
            match reader.read(&mut *buf_temp).await {
              Ok(0) => break 'socket,
              Ok(n) => {
                for i in 0..n {
                  buf.put_u8(buf_temp[i])
                }
              },
              Err(_) => break 'socket,
            }
          }
          
          // // Parse headers
          let header_bytes = buf.split_to(body_start);
          let mut raw_headers = vec![httparse::EMPTY_HEADER; header_count - 1];
          let mut raw_request = httparse::Request::new(&mut raw_headers);
          raw_request.parse(&header_bytes).unwrap();
          
          // Construct headers
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
          
          // Map to a nice API for users to work with          
          let (tx_reader, rx_reader) = tokio::sync::mpsc::channel::<Vec<u8>>(c::buffer::DEFAULT);
          
          let req = Request {
            method: raw_request.method.unwrap().to_string(),
            url: raw_request.path.unwrap().to_string(),
            proto: format!("HTTP/1.{}", raw_request.version.unwrap()),
            headers: Headers::from(headers),
            body_buf: Default::default(),
            body: rx_reader,
            host: Default::default(),
          };
          
          let res = Response {
            head: Default::default(),
            body_buf: Default::default(),
            headers: Default::default(),
            writer: tx_writer.clone(),
          };
          
          // Call the handler function
          // If there is no body, run the handler directly
          if content_length == 0 {
            drop(tx_reader);
            handler(req, res).await.unwrap();
            buf.clear();
            continue;
          }
          
          // Otherwise spawn an async task for the handler
          // and buffer the body as the handler requests it
          tokio::task::spawn(async move {
            handler(req, res).await.unwrap();
          });
          
          // Read bytes for body until Content-Length
          // TODO Transfer-Encoding: chunked
          let mut bytes_read = 0;
          
          loop {
            let mut bytes_to_take = content_length - bytes_read;
            if bytes_to_take > buf.len() {
              bytes_to_take = buf.len();
            }
  
            let bytes = buf.split_to(bytes_to_take);
  
            if bytes.len() > 0 {
              bytes_read += bytes.len();
              if tx_reader.send(bytes.to_vec()).await.is_err() {
                // Handler finished early, drain the buffer and
                // move onto the next request
                continue;
              };
            }
  
            if bytes_read == content_length {
              drop(tx_reader);
              break;
            }
  
            match reader.read(&mut *buf_temp).await {
              Ok(0) => break 'socket,
              Ok(n) => {
                for i in 0..n {
                  buf.put_u8(buf_temp[i])
                }
              },
              Err(_) => break 'socket,
            };
          }
          
          buf.clear();        
        }
      });
    }
  
    self.handler.borrow_mut().replace(handler).unwrap();
    Ok(())
  }
}

fn create_listener<A: ToSocketAddrs>(addr: A) -> io::Result<tokio::net::TcpListener> {
  let mut addrs = addr.to_socket_addrs()?;
  let addr = addrs.next().unwrap();
  let listener = match &addr {
    SocketAddr::V4(_) => Socket::new(Domain::IPV4, Type::STREAM, None)?,
    SocketAddr::V6(_) => Socket::new(Domain::IPV6, Type::STREAM, None)?,
  };

  listener.set_nonblocking(true)?;
  listener.set_nodelay(true)?;
  listener.set_reuse_address(true)?;
  listener.set_linger(Some(Duration::from_secs(0)))?;
  listener.bind(&addr.into())?;
  listener.listen(i32::MAX)?;

  let listener = std::net::TcpListener::from(listener);
  let listener = tokio::net::TcpListener::from_std(listener)?;
  Ok(listener)
}
