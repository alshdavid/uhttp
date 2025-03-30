use tokio::io::AsyncRead;

pub struct Request {
  pub(crate) inner: Box<dyn AsyncRead + Unpin + Send + Sync>,
}

impl Request {
  pub fn body(&mut self) -> &mut Box<dyn AsyncRead + Unpin + Send + Sync> {
    return &mut self.inner;
  }
}
