mod payload;

#[macro_use]
extern crate may;

use bytes::BufMut;
use httparse::{Status};
use may::{
net::TcpListener, sync::{mpsc, spsc::{self, channel, Sender}}
};
use payload::DATA;
use std::{
  io::{BufRead, BufReader, Read, Write},
  net::Shutdown,
  rc::Rc,
  sync::Arc,
  thread,
};
const NL: u8 = b'\n';

type RequestInit = (Vec<u8>, spsc::Receiver<Vec<u8>>, spsc::Sender<Vec<u8>>);

fn main() {
  may::config().set_workers(4);

  let listener = TcpListener::bind("0.0.0.0:8080").unwrap();

  let (tx_init, rx_init) = mpsc::channel::<RequestInit>();

  thread::spawn(move || {
    while let Ok((header_bytes, rx_read, tx_write)) = rx_init.recv() {
      let mut headers = [httparse::EMPTY_HEADER; 16];
      let mut req = httparse::Request::new(&mut headers);
      req.parse(&header_bytes).unwrap();
      tx_write.send(DATA.to_vec()).unwrap();

      let mut b = vec![];
      while let Ok(bytes) = rx_read.recv() {
        b.extend(bytes);
        println!("{}", String::from_utf8(b.clone()).unwrap());
      }

    }
  });

  while let Ok((mut stream, _)) = listener.accept() {
    let tx_init = tx_init.clone();

    go!(move || {
      let mut buf = Vec::<u8>::new();
      let mut tx_read_request = None::<Sender<Vec<u8>>>;

      loop {
        let mut temp_buf = vec![0; 512];
        match stream.read(&mut temp_buf) {
          Ok(n) => {
            if n == 0 {
              break;
            }

            buf.put(&temp_buf[0..n]);
            let mut headers = [httparse::EMPTY_HEADER; 16];
            let mut req = httparse::Request::new(&mut headers);

            if let Ok(Status::Complete(i)) = req.parse(&buf) {
              tx_read_request.take();
              let (tx_read, rx_read) = channel::<Vec<u8>>();
              let (tx_write, rx_write) = channel::<Vec<u8>>();
              
              let (header_bytes, data) = buf.split_at(i);
              tx_init.send((header_bytes.to_vec(), rx_read, tx_write)).unwrap();
              tx_read.send(data.to_vec()).ok();
              tx_read_request = Some(tx_read);

              go!({
                let mut stream = stream.try_clone().unwrap();
                move || {
                  while let Ok(bytes) = rx_write.recv() {
                    if stream.write_all(&bytes).is_err() {
                      return;
                    };
                  }
                }
              });

              continue;
            }

            if let Some(tx_read) = &tx_read_request {
              tx_read.send(temp_buf[0..n].to_vec()).unwrap();
            }
          }
          Err(err) => println!("err = {err:?}"),
        }
      }
    });
  }
}
