mod constants;
mod payload;

#[macro_use]
extern crate may;

use std::collections::HashMap;
use std::io::Read;
use std::io::Write;

use may::net::TcpListener;
use payload::DATA;

use self::constants::{self as c};

const BUF_SIZE: usize = 512;

fn main() {
  std::process::Command::new("clear").status().unwrap();
  may::config().set_workers(4);

  println!("Listening on http://localhost:8080");
  let listener = TcpListener::bind("0.0.0.0:8080").unwrap();

  while let Ok((mut stream, _)) = listener.accept() {
    go!(move || {
      let mut buf = Vec::<u8>::new();

      let mut header_count = 0;
      let mut body_start = 0;
      let mut content_length = 0;
      let mut rc0 = false;
      let mut nl0 = false;
      let mut rc1 = false;

      loop {
        if content_length != 0 && buf.len() >= content_length {
          let _body = buf.drain(0..content_length).collect::<Vec<u8>>();
          body_start = 0;
          header_count = 0;
          content_length = 0;
          stream.write_all(&DATA).unwrap();
        }

        let mut temp_buf = vec![0; BUF_SIZE];
        match stream.read(&mut temp_buf) {
          Ok(n) => {
            if n == 0 {
              break;
            }

            let cursor = buf.len();
            buf.extend(temp_buf.drain(0..n));

            if content_length != 0 {
              continue;
            }

            for i in cursor..(cursor + n) {
              if rc0 == false && buf[i] == c::chars::RC {
                rc0 = true;
              } else if rc0 == true && nl0 == false && buf[i] == c::chars::NL {
                nl0 = true;
                header_count += 1;
              } else if rc0 == true && nl0 == true && rc1 == false && buf[i] == c::chars::RC {
                rc1 = true;
              } else if rc0 == true && nl0 == true && rc1 == true && buf[i] == c::chars::NL {
                body_start = i + 1;
                break;
              } else {
                rc0 = false;
                nl0 = false;
                rc1 = false;
              }
            }

            if body_start == 0 {
              continue;
            }

            let mut raw_headers = vec![httparse::EMPTY_HEADER; header_count - 1];
            let mut raw_request = httparse::Request::new(&mut raw_headers);
            raw_request.parse(&buf).unwrap();

            let mut headers = HashMap::<String, Vec<String>>::new();
            for i in 0..raw_request.headers.len() {
              let mut header =
                std::mem::replace(&mut raw_request.headers[i], httparse::EMPTY_HEADER);
              let header_name = header.name.to_string();
              let header_value = std::mem::take(&mut header.value).to_owned();
              let header_value = unsafe { String::from_utf8_unchecked(header_value) };
              headers.entry(header_name).or_default().push(header_value);
            }

            let Some(method) = raw_request.method else {
              panic!("No method");
            };

            if method == c::methods::GET {
              stream.write_all(&DATA).unwrap();
              body_start = 0;
              header_count = 0;
              content_length = 0;
              buf.clear();
              continue;
            }

            buf = buf.split_off(body_start);

            if let Some(v) = headers.get(c::headers::CONTENT_LENGTH) {
              content_length = v.get(0).unwrap().parse::<usize>().unwrap();
              continue;
            }
          }
          Err(err) => println!("err = {err:?}"),
        }
      }
    });
  }
}
