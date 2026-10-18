use crate::Request;
use crate::headers;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) enum Encoding {
  Zstd,
  Brotli,
  Gzip,
  Identity,
}

impl Encoding {
  /// Value to send back in the `Content-Encoding` header, if any
  pub(crate) fn header_value(&self) -> Option<&'static str> {
    match self {
      Self::Zstd => Some("zstd"),
      Self::Brotli => Some("br"),
      Self::Gzip => Some("gzip"),
      Self::Identity => None,
    }
  }

  /// Mixed into the etag so encodings of the same file don't share a cache entry
  pub(crate) fn etag_suffix(&self) -> &'static str {
    self.header_value().unwrap_or("")
  }
}

pub(crate) struct EncodingResolver;

impl EncodingResolver {
  /// `compress` disabled has no resolver, so "don't compress" is represented as
  /// `None` rather than a separate identity strategy.
  pub(crate) fn new(compress: bool) -> Option<Self> {
    compress.then_some(EncodingResolver)
  }

  /// Pick an encoding to respond with based on the request's `Accept-Encoding`
  pub(crate) fn resolve(
    &self,
    req: &Request,
  ) -> Encoding {
    let Some(accept_encoding) = req.headers.get(headers::ACCEPT_ENCODING) else {
      return Encoding::Identity;
    };

    let Ok(accept_encoding) = accept_encoding.to_str() else {
      return Encoding::Identity;
    };

    if accept_encoding.contains("zstd") {
      Encoding::Zstd
    } else if accept_encoding.contains("br") {
      Encoding::Brotli
    } else if accept_encoding.contains("gz") {
      Encoding::Gzip
    } else {
      Encoding::Identity
    }
  }
}
