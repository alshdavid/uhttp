#[cfg(feature = "router")]
use std::collections::HashMap;
use std::pin::Pin;

use futures::TryStreamExt;
use http::Extensions;
use http::HeaderMap;
use http::HeaderValue;
use http::Method;
use http::Uri;
use http::Version;
use http_body_util::BodyExt;
use serde::de::DeserializeOwned;
use tokio::io::AsyncRead;

use super::types::HttpBody;
use super::types::HttpRequest;
use super::types::HttpStreamReader;

pub struct Request {
  /// The request's method
  pub method: Method,

  /// The request's URI
  pub uri: Uri,

  /// The request's version
  pub version: Version,

  /// The request's headers
  pub headers: HeaderMap<HeaderValue>,

  /// The request's extensions
  pub extensions: Extensions,

  pub body: Pin<Box<dyn AsyncRead + Send>>,

  #[cfg(feature = "router")]
  pub params: HashMap<String, String>,

  /// The route *pattern* this request matched, e.g. `/users/:id` — never the
  /// concrete path.
  ///
  /// Populated by the router after matching. Telemetry should label spans and
  /// metrics with this rather than `uri.path()`, since the concrete path embeds
  /// ids and would create unbounded cardinality. `None` means nothing matched
  /// (the 404 path).
  #[cfg(feature = "router")]
  pub route: Option<String>,
}

impl Default for Request {
  fn default() -> Self {
    Self {
      method: Default::default(),
      uri: Default::default(),
      version: Default::default(),
      headers: Default::default(),
      extensions: Default::default(),
      body: Box::pin(tokio::io::empty()),
      #[cfg(feature = "router")]
      params: Default::default(),
      #[cfg(feature = "router")]
      route: None,
    }
  }
}

impl std::fmt::Debug for Request {
  fn fmt(
    &self,
    f: &mut std::fmt::Formatter<'_>,
  ) -> std::fmt::Result {
    let mut debug = f.debug_struct("Request");
    debug
      .field("method", &self.method)
      .field("uri", &self.uri)
      .field("version", &self.version)
      .field("headers", &self.headers)
      .field("extensions", &self.extensions)
      .field("body", &"<body>".to_string());
    #[cfg(feature = "router")]
    debug
      .field("params", &self.params)
      .field("route", &self.route);
    debug.finish()
  }
}

impl Request {
  pub fn new<B>(request: HttpRequest<B>) -> Self
  where
    B: 'static + Send + HttpBody,
    B::Data: Send,
    B::Error: Into<Box<dyn std::error::Error + Send + Sync>>,
  {
    let (parts, body) = request.into_parts();

    let body = HttpStreamReader::new(body.into_data_stream().map_err(std::io::Error::other));

    Self {
      method: parts.method,
      uri: parts.uri,
      version: parts.version,
      headers: parts.headers,
      extensions: parts.extensions,
      #[cfg(feature = "router")]
      params: HashMap::new(),
      body: Box::pin(body),
      // Set by the router once a route matches; see the field docs.
      #[cfg(feature = "router")]
      route: None,
    }
  }

  pub fn query<T: DeserializeOwned>(&self) -> crate::Result<T> {
    let Some(query_str) = self.uri.query() else {
      return Err(anyhow::anyhow!("No query string"));
    };

    match serde_urlencoded::from_str::<T>(query_str) {
      Ok(query) => Ok(query),
      Err(err) => Err(anyhow::anyhow!("{:?}", err)),
    }
  }
}

impl<B> From<HttpRequest<B>> for Request
where
  B: 'static + Send + HttpBody,
  B::Data: Send,
  B::Error: Into<Box<dyn std::error::Error + Send + Sync>>,
{
  fn from(value: HttpRequest<B>) -> Self {
    Self::new(value)
  }
}
