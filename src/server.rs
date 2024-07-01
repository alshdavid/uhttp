use std::io;

use super::Request;
use super::Response;

pub struct Server {}

impl Server {
  pub fn new(_handler: impl Fn(Request, Box<dyn Response>) -> io::Result<()>) -> Self {
    Self {}
  }

  pub fn listen(
    &self,
    _address: &str,
    _port: u16,
  ) -> Result<(), ()> {
    Ok(())
  }
}
