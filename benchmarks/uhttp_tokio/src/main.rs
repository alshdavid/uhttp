mod constants;
mod payload;

#[macro_use]
extern crate may;

use std::collections::HashMap;
use std::io::Read;
use std::io::Write;

use may::net::TcpListener;
use may::sync::spsc;
use payload::DATA;

use self::constants::{self as c};

const BUF_SIZE: usize = 512;

fn handler(
  headers: HashMap<String, Vec<String>>,
  rx_read: spsc::Receiver<Vec<u8>>,
  tx_write: spsc::Sender<Vec<u8>>,
) {
  let mut b = vec![];
  while let Ok(bytes) = rx_read.recv() {
    b.extend(bytes);
  }

  // println!("{}", String::from_utf8(b).unwrap());

  tx_write.send(DATA.to_vec()).unwrap()
}

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
        let mut temp_buf = vec![0; BUF_SIZE];
        match stream.read(&mut temp_buf) {
          Ok(n) => {
            if n == 0 {
              break;
            }

            let cursor = buf.len();
            buf.extend(temp_buf.drain(0..n));

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
          }
          Err(err) => println!("err = {err:?}"),
        }

        if body_start == 0 {
          continue;
        }

        let mut raw_headers = vec![httparse::EMPTY_HEADER; header_count - 1];
        let mut raw_request = httparse::Request::new(&mut raw_headers);
        raw_request.parse(&buf).unwrap();

        let mut headers = HashMap::<String, Vec<String>>::new();
        for i in 0..raw_request.headers.len() {
          let mut header = std::mem::replace(&mut raw_request.headers[i], httparse::EMPTY_HEADER);
          let header_name = header.name.to_string();
          let header_value = std::mem::take(&mut header.value).to_owned();
          let header_value = unsafe { String::from_utf8_unchecked(header_value) };
          headers.entry(header_name).or_default().push(header_value);
        }

        let Some(method) = raw_request.method else {
          panic!("No method");
        };

        if let Some(v) = headers.get(c::headers::CONTENT_LENGTH) {
          content_length = v.get(0).unwrap().parse::<usize>().unwrap();
        } else {
          content_length = 0;
        }

        let (tx_read, rx_read) = spsc::channel::<Vec<u8>>();
        let (tx_write, rx_write) = spsc::channel::<Vec<u8>>();

        go!(move || {
          handler(headers, rx_read, tx_write);
        });

        go!({
          let mut stream = stream.try_clone().unwrap();
          move || {
            while let Ok(bytes) = rx_write.recv() {
              stream.write_all(&bytes).unwrap();
            }
          }
        });

        if method == c::methods::GET {
          drop(tx_read);
          body_start = 0;
          header_count = 0;
          content_length = 0;
          buf.clear();
          continue;
        }

        buf = buf.split_off(body_start);
        let mut bytes_read = 0;

        loop {
          if bytes_read >= content_length {
            let bytes = buf
              .drain(0..content_length - bytes_read)
              .collect::<Vec<u8>>();
            tx_read.send(bytes).unwrap();
            drop(tx_read);
            body_start = 0;
            header_count = 0;
            content_length = 0;
            break;
          } else if buf.len() > 0 && bytes_read <= content_length {
            let bytes = buf.drain(0..).collect::<Vec<u8>>();
            bytes_read += bytes.len();
            tx_read.send(bytes).unwrap();
            continue;
          }

          let mut temp_buf = vec![0; BUF_SIZE];
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
}
