#[macro_use]
extern crate may;

use bytes::BufMut;
use httparse::{Header, Status};
use may::{
  net::TcpListener,
  sync::{mpsc, spsc},
};
use std::{io::{BufRead, BufReader, Read, Write}, net::Shutdown};

const PAYLOAD: &str = "hello world";
const NL: u8 = b'\n';

const DATA: &[u8] = b"\
  HTTP/1.1 200 OK\r\n\
  Content-Type: text/plain\r\n\
  \r\n\
  hello world
";

type RequestInit = (spsc::Receiver<Vec<u8>>, mpsc::Sender<Vec<u8>>);

fn main() {
  let listener = TcpListener::bind("0.0.0.0:8080").unwrap();

  let mut senders = Vec::<mpsc::Sender<RequestInit>>::new();

  for _ in 0..16 {
    let (tx_headers, rx_headers) = mpsc::channel::<RequestInit>();
    senders.push(tx_headers);

    std::thread::spawn(move || {
      while let Ok((rx_read, tx_write)) = rx_headers.recv() {

        rx_read.recv();

        tx_write
          .send(
            format!(
              "HTTP/1.1 200 OK\r\nDate: 1-1-2000\r\n\r\nhello world",
            )
            .as_bytes()
            .to_vec(),
          )
          .unwrap();
      }
    });
  }

  let mut current = 0;

  while let Ok((mut stream, _)) = listener.accept() {
    let tx_init = senders[current].clone();
    if current == senders.len() - 1 {
      current = 0;
    } else {
      current += 1;
    }

    let (tx_write, rx_write) = mpsc::channel::<Vec<u8>>();
    let (tx_read, rx_read) = spsc::channel::<Vec<u8>>();

    tx_init.send((rx_read, tx_write)).unwrap();

    go!({
      let mut stream = stream.try_clone().unwrap();
      move || {
        if let Ok(bytes) = (&rx_write).recv() {
          stream.write_all(&bytes).unwrap();
        }
        // stream.shutdown(Shutdown::Write).unwrap();
      }
    });

    go!(move || {
      loop {
        let mut temp_buf = vec![0; 512];
        match stream.read(&mut temp_buf) {
          Ok(n) => {
            if n == 0 || n < 512{
              // Connection was closed
              // println!("con closed");
              break;
            }
            if tx_read.send(temp_buf[0..n].to_vec()).is_err() {
              // Sender closed
              // stream.shutdown(Shutdown::Read).unwrap();
              // println!("sender closed");

              continue;;
            };
          }
          Err(err) => {
            println!("err = {err:?}")
          }
        }
      }
    });
  }
}
