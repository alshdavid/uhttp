use std::any::Any;

use http::Extensions;
use http::HeaderMap;
use http::HeaderName;
use http::HeaderValue;
use http::StatusCode;
use http::Version;
#[cfg(feature = "json")]
use serde::Serialize;

use super::types::HyperResponseBuilder;
use crate::ResponseBody;
#[cfg(feature = "json")]
use crate::headers;

pub struct Response(HyperResponseBuilder);

impl Response {
  pub(crate) fn new() -> Self {
    Self(super::types::HyperResponse::builder())
  }

  pub fn status<T>(
    self,
    status: T,
  ) -> Self
  where
    T: TryInto<StatusCode>,
    <T as TryInto<StatusCode>>::Error: Into<http::Error>,
  {
    Self(self.0.status(status))
  }

  pub fn version(
    self,
    version: Version,
  ) -> Self {
    Self(self.0.version(version))
  }

  pub fn header<K, V>(
    self,
    key: K,
    value: V,
  ) -> Self
  where
    K: TryInto<HeaderName>,
    <K as TryInto<HeaderName>>::Error: Into<http::Error>,
    V: TryInto<HeaderValue>,
    <V as TryInto<HeaderValue>>::Error: Into<http::Error>,
  {
    Self(self.0.header(key, value))
  }

  pub fn headers_ref(&self) -> Option<&HeaderMap<HeaderValue>> {
    self.0.headers_ref()
  }

  pub fn headers_mut(&mut self) -> Option<&mut HeaderMap<HeaderValue>> {
    self.0.headers_mut()
  }

  pub fn extension<T>(
    self,
    extension: T,
  ) -> Self
  where
    T: Clone + Any + Send + Sync + 'static,
  {
    Self(self.0.extension(extension))
  }

  pub fn extensions_ref(&self) -> Option<&Extensions> {
    self.0.extensions_ref()
  }

  pub fn extensions_mut(&mut self) -> Option<&mut Extensions> {
    self.0.extensions_mut()
  }

  pub fn body(
    self,
    bytes: impl Into<ResponseBody>,
  ) -> crate::Result<super::types::HyperInfallibleResponse> {
    let response = match bytes.into() {
      ResponseBody::Bytes(bytes) => {
        let content = super::types::HyperBytes::from(bytes);
        self
          .0
          .body(super::types::HttpBoxBody::new(super::types::HttpFull::new(
            content,
          )))
      }
      ResponseBody::BoxedBody(body) => self.0.body(body),
    };
    Ok(response?)
  }

  /// Build a response with an empty body.
  pub fn empty(self) -> crate::Result<super::types::HyperInfallibleResponse> {
    self.body(vec![])
  }

  /// Build a JSON response, setting the `Content-Type` header to
  /// `application/json`.
  #[cfg(feature = "json")]
  pub fn json(
    self,
    json: impl Serialize,
  ) -> crate::Result<super::types::HyperInfallibleResponse> {
    self
      .header(headers::CONTENT_TYPE, "application/json")
      .body(serde_json::to_vec(&json)?)
  }
}
