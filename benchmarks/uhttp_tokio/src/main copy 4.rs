mod payload;

use payload::DATA;

use std::net::SocketAddr;

use futures::{executor::{LocalPool, LocalSpawner}, task::{LocalSpawnExt, SpawnExt}};
use async_std::{io::{ReadExt, WriteExt}, net::TcpListener};

fn main() {
  let mut local_pool = LocalPool::new();
  let spawner2 = local_pool.spawner();
  let spawner = local_pool.spawner();

  spawner.spawn_local(async move {
    let addr = SocketAddr::from(([127, 0, 0, 1], 8080));
    let listener = TcpListener::bind(addr).await.unwrap();

    while let Ok((mut stream, _)) = listener.accept().await {
      spawner2.spawn_local(async move  {
        stream.read(&mut vec![0; 512]).await.unwrap();
        stream.write_all(&DATA).await.unwrap();
      }).unwrap();
    }
  }).unwrap();

  local_pool.run()
}