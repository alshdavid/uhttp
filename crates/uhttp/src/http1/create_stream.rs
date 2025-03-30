use std::convert::Infallible;

use futures::TryStreamExt;
use http_body_util::combinators::BoxBody;
use http_body_util::StreamBody;
use tokio::io::DuplexStream;

use super::internal_types::HyperBytes;
use super::internal_types::ResponseBuilder;

pub(crate) fn create_stream(
  res: ResponseBuilder
) -> crate::Result<(
  http::Response<BoxBody<HyperBytes, Infallible>>,
  DuplexStream,
)> {
  let (writer, reader) = tokio::io::duplex(512);

  let reader_stream = tokio_util::io::ReaderStream::new(reader)
    .map_ok(hyper::body::Frame::data)
    .map_err(|_item| panic!());

  let stream_body = StreamBody::new(reader_stream);

  let boxed_body = BoxBody::<HyperBytes, Infallible>::new(stream_body);

  let res: http::Response<BoxBody<HyperBytes, Infallible>> = res.body(boxed_body)?;

  Ok((res, writer))
}
