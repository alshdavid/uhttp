use std::io;
use std::io::Write;

use may::sync::mpsc;

pub enum ResponseWriteAction {
  Write(Vec<u8>),
  WriteAll(Vec<u8>),
  Flush,
}

pub struct ResponseWriter {
  tx_write: mpsc::Sender<ResponseWriteAction>,
}

impl ResponseWriter {
  pub fn new(tx_write: mpsc::Sender<ResponseWriteAction>) -> Self {
    Self { tx_write }
  }
}

impl Write for ResponseWriter {
  fn write(
    &mut self,
    buf: &[u8],
  ) -> io::Result<usize> {
    if self
      .tx_write
      .send(ResponseWriteAction::Write(buf.to_vec()))
      .is_err()
    {
      return Err(io::Error::other("Write error"));
    }
    Ok(buf.len())
  }

  fn write_all(
    &mut self,
    buf: &[u8],
  ) -> io::Result<()> {
    if self
      .tx_write
      .send(ResponseWriteAction::WriteAll(buf.to_vec()))
      .is_err()
    {
      return Err(io::Error::other("WriteAll error"));
    }
    Ok(())
  }

  fn flush(&mut self) -> io::Result<()> {
    if self.tx_write.send(ResponseWriteAction::Flush).is_err() {
      return Err(io::Error::other("Flush error"));
    }
    Ok(())
  }
}
