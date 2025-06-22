use futures::StreamExt;
use http_body_util::BodyDataStream;
use tokio_util::io::StreamReader;

use super::internal_types::HyperIncoming;
use super::internal_types::HyperRequest;
use crate::Request;

impl From<HyperRequest<HyperIncoming>> for Request {
  fn from(mut req: HyperRequest<HyperIncoming>) -> Self {
    let headers = std::mem::take(req.headers_mut());
    let method = std::mem::take(req.method_mut());
    let version = std::mem::take(req.version_mut());
    let uri = std::mem::take(req.uri_mut());

    let body = req.into_body();
    let body = BodyDataStream::new(body);
    let body_stream = body.map(|result| result.map_err(std::io::Error::other));
    let read = StreamReader::new(body_stream);

    Self {
      inner: Box::new(read),
      headers,
      method,
      version,
      uri,
    }
  }
}
