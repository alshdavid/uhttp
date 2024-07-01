use std::io::Write;

use super::Headers;

pub trait Response: Write {
  fn headers(&self) -> &mut Headers;
  fn write_header(
    &self,
    status_code: usize,
  );
  fn flush(&self);
}
