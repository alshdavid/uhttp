use crate::Request;

#[derive(Clone, Copy, Debug)]
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

pub(crate) fn negotiate_encoding(
  req: &Request,
  compress: bool,
) -> Encoding {
  if !compress {
    return Encoding::Identity;
  }

  let Some(accept_encoding) = req.headers.get("Accept-Encoding") else {
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
