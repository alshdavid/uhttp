#[macro_use]
extern crate may;

use bytes::BufMut;
use httparse::{Header, Status};
use may::{
  net::TcpListener,
  sync::{mpsc, spsc},
};
use std::{io::{BufRead, BufReader, Read, Write}, net::Shutdown, rc::Rc, sync::Arc};

const PAYLOAD: &str = "hello world";
const NL: u8 = b'\n';

type RequestInit = (spsc::Receiver<Vec<u8>>, spsc::Sender<Vec<u8>>);

fn main() {
  may::config().set_workers(4);

  let listener = TcpListener::bind("0.0.0.0:8080").unwrap();

  // let mut senders = Vec::<Arc<mpsc::Sender<RequestInit>>>::new();

  // for _ in 0..100 {
  //   let (tx_headers, rx_headers) = mpsc::channel::<RequestInit>();
  //   senders.push(Arc::new(tx_headers));

  //   std::thread::spawn(move || {
  //     while let Ok((rx_read, tx_write)) = rx_headers.recv() {

  //       tx_write
  //         .send(
  //           format!("HTTP/1.1 200 OK\r\nDate: 1-1-2000\r\n\r\nhello world")
  //           .as_bytes()
  //           .to_vec(),
  //         )
  //         .unwrap();

  //         // rx_read.recv();

  //     }
  //   });
  // }

  // let mut current = 0;

  while let Ok((mut stream, _)) = listener.accept() {
    // let tx_init = senders[current].clone();
    // if current == senders.len() - 1 {
      // current = 0;
    // } else {
      // current += 1;
    // }

    go!(move || {
      // let (tx_write, rx_write) = spsc::channel::<Vec<u8>>();
      // let (tx_read, rx_read) = spsc::channel::<Vec<u8>>();
  
      // tx_init.send((rx_read, tx_write)).unwrap();

      // go!({
      //   let mut stream = stream.try_clone().unwrap();
      //   move || {
      //     while let Ok(bytes) = (&rx_write).recv() {
      //       stream.write_all(&bytes).unwrap();
      //     }
      //     // stream.shutdown(Shutdown::Write).unwrap();
      //   }
      // });



      loop {
        let mut temp_buf = vec![0; 512];
        match stream.read(&mut temp_buf) {
            Ok(n) => break, 
            // Ok(0) => break,
            // Ok(n) => continue, 
            Err(err) => println!("err = {err:?}"),
        }
      }

      stream.write_all(b"HTTP/1.1 200 OK\r\ncontent-length: 13\r\n\r\nHello, World!").unwrap();

      // loop {
      //   let mut temp_buf = vec![0; 512];
      //   match stream.read(&mut temp_buf) {
      //     Ok(n) => {
      //       if n == 0 || n < 512{
      //         // Connection was closed
      //         // println!("con closed");
      //         break;
      //       }

      //       // if tx_read.send(temp_buf[0..n].to_owned()).is_err() {
      //       //   // Sender closed
      //       //   // stream.shutdown(Shutdown::Read).unwrap();
      //       //   println!("sender closed");

      //       //   continue;
      //       // };
      //     }
      //     Err(err) => {
      //       println!("err = {err:?}")
      //     }
      //   }
      // }
    });

  }
}
