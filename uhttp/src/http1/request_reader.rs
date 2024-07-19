use std::io;

use may::sync::spsc;

pub struct RequestReader {
  inner: Vec<u8>,
  rx_read: spsc::Receiver<Vec<u8>>,
}

impl RequestReader {
  pub fn new(rx_read: spsc::Receiver<Vec<u8>>) -> Self {
    Self {
      inner: vec![],
      rx_read,
    }
  }
}

impl io::Read for RequestReader {
  fn read(
    &mut self,
    buf: &mut [u8],
  ) -> io::Result<usize> {
    let inner_len = self.inner.len();
    let buf_len = buf.len();
    let mut cursor = 0;

    if inner_len != 0 {
      cursor = transfer(&mut self.inner, buf, 0);
    }

    if cursor >= buf_len {
      return Ok(cursor);
    }

    if let Ok(mut bytes) = self.rx_read.recv() {
      if bytes.len() == 0 {
        return Ok(cursor);
      }
      cursor = transfer(&mut bytes, buf, cursor);
      self.inner = bytes;
      return Ok(cursor);
    }

    return Ok(cursor);
  }
}

pub fn transfer(
  src: &mut Vec<u8>,
  dest: &mut [u8],
  start: usize,
) -> usize {
  let src_len = src.len();
  let dest_len = dest.len();
  let mut available = dest_len - start;

  if available > src_len {
    available = src_len
  }

  for (i, byte) in src.drain(..available).enumerate() {
    dest[i + start] = byte;
  }

  return available;
}
