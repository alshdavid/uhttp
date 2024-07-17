
/*

  Success rate: 99.99%
  Total:        0.1732 secs
  Slowest:      0.0042 secs
  Fastest:      0.0002 secs
  Average:      0.0009 secs
  Requests/sec: 57746.9573

  Total data:   107.41 KiB
  Size/request: 11 B
  Size/sec:     620.27 KiB

Response time histogram:
  0.000 [1]    |
  0.001 [1256] |■■■■■
  0.001 [7899] |■■■■■■■■■■■■■■■■■■■■■■■■■■■■■■■■
  0.001 [508]  |■■
  0.002 [142]  |
  0.002 [63]   |
  0.003 [39]   |
  0.003 [24]   |
  0.003 [24]   |
  0.004 [28]   |
  0.004 [15]   |

Response time distribution:
  10.00% in 0.0006 secs
  25.00% in 0.0007 secs
  50.00% in 0.0008 secs
  75.00% in 0.0009 secs
  90.00% in 0.0010 secs
  95.00% in 0.0013 secs
  99.00% in 0.0025 secs
  99.90% in 0.0040 secs
  99.99% in 0.0042 secs


Details (average, fastest, slowest):
  DNS+dialup:   0.0001 secs, 0.0000 secs, 0.0007 secs
  DNS-lookup:   0.0000 secs, 0.0000 secs, 0.0003 secs

Status code distribution:
  [200] 9999 responses

*/

// mod uhttp;

// use bytes::{BufMut, BytesMut};
// use mio;
// use std::{
//   io::{self, BufRead, BufReader, Write}, net::{SocketAddr, TcpListener, TcpStream}, os::fd::AsFd, sync::mpsc::{channel, Sender}, thread
// };

// const BUF_LEN: usize = 4096 * 8;
// pub const RC: u8 = b'\r';
// pub const NL: u8 = b'\n';

// fn main() {
//   let addr = SocketAddr::from(([127, 0, 0, 1], 8080));
//   let listener = TcpListener::bind(addr).unwrap();

//   let mut senders = Vec::<Sender<TcpStream>>::new();

//   for _ in 0..1000 {
//     let (tx, rx) = channel::<TcpStream>();
//     senders.push(tx);

//     thread::spawn(move || -> io::Result<()> {
//       let mut poll = mio::Poll::new().unwrap();
//       let mut events = mio::Events::with_capacity(128);
       
//       while let Ok(mut stream) = rx.recv() {
//         let fd = stream.as_fd();
//         // stream.set_nonblocking(true)?;

//         mio::Waker::
//         poll.registry().register(&mut stream, token, interests);


//         // let mut header_bytes = [0u8; BUF_LEN];
//         let mut header_bytes = Vec::<u8>::new();

//         let mut reader = BufReader::new(stream.try_clone()?);

//         loop {
//           let read = reader.read_until(NL, &mut header_bytes)?;
//           if read == 2 {
//             break;
//           }
//         }

//         let mut headers = [httparse::EMPTY_HEADER; 16];
//         let mut r = httparse::Request::new(&mut headers);
//         r.parse(&header_bytes).unwrap();

//         stream.write(
//           b"\
//           HTTP/1.1 200 OK\r\n\
//           Content-Type: text/plain\r\n\
//           \r\n\
//           hello world
//         ")?;
//       }

//       Ok(())
//     });
//   }

//   let mut current = 0;

//   while let Ok((stream, _)) = listener.accept() {
//     senders[current].send(stream).unwrap();
//     if current == senders.len() - 1 {
//       current = 0;
//     } else {
//       current += 1;
//     }
//   }
// }
