mod payload;

use std::{fmt::format, net::SocketAddr};

use once_cell::sync::Lazy;
use payload::DATA;
use tokio::{io::{AsyncReadExt, AsyncWriteExt}, net::TcpListener, runtime::Builder};

fn main() {
  
  let rt = Builder::new_multi_thread()
    .worker_threads(16)
    .enable_all()
    .build()
    .unwrap();
  
  rt.block_on({
    let handle = rt.handle();
    
    async move {
      let addr = SocketAddr::from(([0, 0, 0, 0], 8080));
      let listener = TcpListener::bind(addr).await.unwrap();

      while let Ok((mut stream, _)) = listener.accept().await {
        handle.spawn(async move {
          stream.read(&mut vec![0; 512]).await.unwrap();
          stream.write_all(&DATA).await.unwrap();

          // stream.read(&mut vec![0; 512]).await.unwrap();
        });
      }
    }
  });
}