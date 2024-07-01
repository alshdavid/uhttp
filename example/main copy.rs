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

    let mut request_headers = vec![httparse::EMPTY_HEADER; header_count - 1];
    let mut req = httparse::Request::new(&mut request_headers);
    req.parse(&header_bytes).map_err(|e| io::Error::other(e))?;


    dbg!(&req);

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
