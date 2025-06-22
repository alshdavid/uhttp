use http::HeaderMap;
use http::Method;
use http::Uri;
use http::Version;
use tokio::io::AsyncRead;

pub struct Request {
  pub(crate) inner: Box<dyn AsyncRead + Unpin + Send + Sync>,
  pub(crate) headers: HeaderMap,
  pub(crate) method: Method,
  pub(crate) version: Version,
  pub(crate) uri: Uri,
}

impl Request {
  pub fn body(&mut self) -> &mut Box<dyn AsyncRead + Unpin + Send + Sync> {
    &mut self.inner
  }

  pub fn headers(&self) -> &HeaderMap {
    &self.headers
  }

  pub fn method(&self) -> &Method {
    &self.method
  }

  pub fn version(&self) -> &Version {
    &self.version
  }

  pub fn uri(&self) -> &Uri {
    &self.uri
  }
}
