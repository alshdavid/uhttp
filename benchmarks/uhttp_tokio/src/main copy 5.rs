mod payload;

use std::net::SocketAddr;
use async_std::{io::{ReadExt, WriteExt}, net::TcpListener, task::{Builder, spawn}};

use payload::DATA;

fn main() {
  let rt = Builder::new();

  rt.blocking(async move {
    let addr = SocketAddr::from(([127, 0, 0, 1], 8080));
    let listener = TcpListener::bind(addr).await.unwrap();

    while let Ok((mut stream, _)) = listener.accept().await {
      spawn(async move {
        stream.read(&mut vec![0; 512]).await.unwrap();
        stream.write_all(&DATA).await.unwrap();
      });
    }
  });
}