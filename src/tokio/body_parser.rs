use std::io;

use crate::constants::DEFAULT_BUFFER_SIZE;
use crate::BodyReaderRef;

pub async fn bytes(reader: &mut BodyReaderRef) -> io::Result<Vec<u8>> {
  let mut body = Vec::<u8>::new();
  let mut buf = [0u8; DEFAULT_BUFFER_SIZE];

  loop {
    let count = reader.read(&mut buf).await?;
    body.extend(buf);
    if count == 0 {
      break;
    }
  }

  Ok(body)
}

pub async fn utf8(reader: &mut BodyReaderRef) -> io::Result<String> {
  let body = bytes(reader).await?;
  String::from_utf8(body).map_err(|err| io::Error::other(err))
}

pub async unsafe fn utf8_unchecked(reader: &mut BodyReaderRef) -> io::Result<String> {
  let body = bytes(reader).await?;
  Ok(unsafe { String::from_utf8_unchecked(body) })
}
