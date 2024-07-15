mod uhttp;

use bytes::{BufMut, BytesMut};
use memchr;
use std::{
  io::{self, BufRead, BufReader, Write},
  net::{SocketAddr, TcpListener, TcpStream},
  sync::mpsc::{channel, Sender},
  thread,
};

pub const RC: u8 = b'\r';
pub const NL: u8 = b'\n';

fn main() {
  let addr = SocketAddr::from(([127, 0, 0, 1], 8080));
  let listener = TcpListener::bind(addr).unwrap();

  let mut senders = Vec::<Sender<TcpStream>>::new();

  for _ in 0..1000 {
    let (tx, rx) = channel::<TcpStream>();
    senders.push(tx);

    thread::spawn(move || -> io::Result<()> {
      while let Ok(mut stream) = rx.recv() {
        let mut header_bytes = Vec::<u8>::new();
        let mut reader = BufReader::new(stream.try_clone()?);

        loop {
          let read = reader.read_until(NL, &mut header_bytes)?;
          if read == 2 {
            break;
          }
        }

        let mut headers = [httparse::EMPTY_HEADER; 16];
        let mut r = httparse::Request::new(&mut headers);
        r.parse(&header_bytes).unwrap();

        stream.write(
          b"\
          HTTP/1.1 200 OK\r\n\
          Content-Type: text/plain\r\n\
          \r\n\
          hello world
        ")?;
      }

      Ok(())
    });
  }

  let mut current = 0;

  while let Ok((stream, _)) = listener.accept() {
    senders[0].send(stream).unwrap();
    if current == senders.len() - 1 {
      current = 0;
    } else {
      current += 1;
    }
  }
}
