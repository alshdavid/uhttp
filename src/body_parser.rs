use std::io;
use std::io::Read;

use crate::constants::DEFAULT_BUFFER_SIZE;

pub fn bytes(reader: &mut impl Read) -> io::Result<Vec<u8>> {
  let mut body = Vec::<u8>::new();
  let mut buf = [0u8; DEFAULT_BUFFER_SIZE];

  loop {
    let count = reader.read(&mut buf)?;
    body.extend(buf);
    if count == 0 {
      break;
    }
  }
  Ok(body)
}

pub fn utf8(reader: &mut impl Read) -> io::Result<String> {
  let body = bytes(reader)?;
  String::from_utf8(body).map_err(|err| io::Error::other(err))
}

pub unsafe fn utf8_unchecked(reader: &mut impl Read) -> io::Result<String> {
  let body = bytes(reader)?;
  Ok(unsafe { String::from_utf8_unchecked(body) })
}
