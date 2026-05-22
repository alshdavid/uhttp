use crate::HandleFunc;

pub struct FileServerOptions {}

pub fn handler() -> HandleFunc {
  Box::new(|_req, _res| {
    Box::pin(async move {
      Ok(())
    })
  })
}