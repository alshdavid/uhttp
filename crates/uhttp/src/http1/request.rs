use futures::StreamExt;
use http_body_util::BodyDataStream;
use tokio_util::io::StreamReader;

use super::internal_types::HyperIncoming;
use super::internal_types::HyperRequest;
use crate::Request;

impl From<HyperRequest<HyperIncoming>> for Request {
  fn from(req: HyperRequest<HyperIncoming>) -> Self {
    let body = req.into_body();
    let body = BodyDataStream::new(body);
    let body_stream = body.map(|result| result.map_err(|error| std::io::Error::other(error)));
    let read = StreamReader::new(body_stream);

    Self {
      inner: Box::new(read),
    }
  }
}
