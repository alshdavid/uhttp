use std::io;

use crate::c;
use crate::HttpReader;

pub async fn bytes(reader: &mut dyn HttpReader) -> io::Result<Vec<u8>> {
  let mut body = Vec::<u8>::new();
  let mut buf = vec![0u8; c::buffer::DEFAULT];

  loop {
    let count = reader.read(&mut buf).await?;
    body.extend(buf.drain(..count));
    if count == 0 {
      break;
    }
  }

  Ok(body)
}

pub async fn utf8(reader: &mut dyn HttpReader) -> io::Result<String> {
  let body = bytes(reader).await?;
  String::from_utf8(body).map_err(|err| io::Error::other(err))
}

pub async unsafe fn utf8_unchecked(reader: &mut dyn HttpReader) -> io::Result<String> {
  let body = bytes(reader).await?;
  Ok(unsafe { String::from_utf8_unchecked(body) })
}
