use std::collections::HashMap;

use crate::headers;

/// Utility for building commonly used custom response headers.
///
/// Useful for configuring a [`crate::file_server::FileServerOptions`] with
/// sensible security and cross-origin defaults.
///
/// ```rust
/// use uhttp::file_server::CrossOriginEmbedderPolicy;
/// use uhttp::file_server::CrossOriginOpenerPolicy;
/// use uhttp::file_server::CustomHeaders;
/// use uhttp::file_server::FrameOptions;
///
/// let headers = CustomHeaders::builder()
///   .access_control_allow_origin("*")
///   .access_control_allow_methods("GET, POST")
///   .frame_options(FrameOptions::SameOrigin)
///   .content_security_policy("default-src 'self'")
///   .cross_origin_opener_policy(CrossOriginOpenerPolicy::SameOrigin)
///   .cross_origin_embedder_policy(CrossOriginEmbedderPolicy::RequireCorp)
///   .build();
/// ```
pub struct CustomHeaders;

impl CustomHeaders {
  /// Start building a set of custom headers
  pub fn builder() -> CustomHeadersBuilder {
    CustomHeadersBuilder::default()
  }
}

/// Accepted values for the `X-Frame-Options` header
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FrameOptions {
  /// The page cannot be displayed in a frame
  Deny,
  /// The page can only be displayed in a frame on the same origin
  SameOrigin,
}

impl FrameOptions {
  fn as_value(&self) -> &'static str {
    match self {
      FrameOptions::Deny => "DENY",
      FrameOptions::SameOrigin => "SAMEORIGIN",
    }
  }
}

/// Accepted values for the `Cross-Origin-Opener-Policy` header
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CrossOriginOpenerPolicy {
  /// Isolate the browsing context to same-origin documents
  SameOrigin,
  /// Keep the browsing context open for same-origin documents and popups
  SameOriginAllowPopups,
  /// Do not isolate the browsing context
  UnsafeNone,
}

impl CrossOriginOpenerPolicy {
  fn as_value(&self) -> &'static str {
    match self {
      CrossOriginOpenerPolicy::SameOrigin => "same-origin",
      CrossOriginOpenerPolicy::SameOriginAllowPopups => "same-origin-allow-popups",
      CrossOriginOpenerPolicy::UnsafeNone => "unsafe-none",
    }
  }
}

/// Accepted values for the `Cross-Origin-Embedder-Policy` header
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CrossOriginEmbedderPolicy {
  /// Require cross-origin resources to opt in via CORP
  RequireCorp,
  /// Do not require cross-origin resources to opt in
  UnsafeNone,
}

impl CrossOriginEmbedderPolicy {
  fn as_value(&self) -> &'static str {
    match self {
      CrossOriginEmbedderPolicy::RequireCorp => "require-corp",
      CrossOriginEmbedderPolicy::UnsafeNone => "unsafe-none",
    }
  }
}

/// Builder for a set of custom response headers
#[derive(Debug, Default)]
pub struct CustomHeadersBuilder {
  headers: HashMap<String, String>,
}

impl CustomHeadersBuilder {
  /// Add a header using an arbitrary name and value
  pub fn header(
    mut self,
    key: impl Into<String>,
    value: impl Into<String>,
  ) -> Self {
    self.headers.insert(key.into(), value.into());
    self
  }

  /// `Access-Control-Allow-Origin`
  ///
  /// Use `"*"` to allow any origin, or a specific origin such as
  /// `"https://example.com"`
  pub fn access_control_allow_origin(
    self,
    value: impl Into<String>,
  ) -> Self {
    self.header(headers::ACCESS_CONTROL_ALLOW_ORIGIN, value)
  }

  /// `Access-Control-Allow-Methods`
  ///
  /// A comma separated list of permitted methods, eg `"GET, POST"`
  pub fn access_control_allow_methods(
    self,
    value: impl Into<String>,
  ) -> Self {
    self.header(headers::ACCESS_CONTROL_ALLOW_METHODS, value)
  }

  /// `X-Frame-Options`
  pub fn frame_options(
    self,
    value: FrameOptions,
  ) -> Self {
    self.header(headers::X_FRAME_OPTIONS, value.as_value())
  }

  /// `Content-Security-Policy`
  pub fn content_security_policy(
    self,
    value: impl Into<String>,
  ) -> Self {
    self.header(headers::CONTENT_SECURITY_POLICY, value)
  }

  /// `Cross-Origin-Opener-Policy`
  pub fn cross_origin_opener_policy(
    self,
    value: CrossOriginOpenerPolicy,
  ) -> Self {
    self.header(headers::CROSS_ORIGIN_OPENER_POLICY, value.as_value())
  }

  /// `Cross-Origin-Embedder-Policy`
  pub fn cross_origin_embedder_policy(
    self,
    value: CrossOriginEmbedderPolicy,
  ) -> Self {
    self.header(headers::CROSS_ORIGIN_EMBEDDER_POLICY, value.as_value())
  }

  /// Collect the configured headers into a [`HashMap`]
  pub fn build(self) -> HashMap<String, String> {
    self.headers
  }
}

#[cfg(test)]
mod tests {
  use std::collections::HashMap;

  use super::CrossOriginEmbedderPolicy;
  use super::CrossOriginOpenerPolicy;
  use super::CustomHeaders;
  use super::FrameOptions;
  use crate::headers;

  #[test]
  fn builds_a_map_of_configured_headers() {
    let headers = CustomHeaders::builder()
      .access_control_allow_origin("*")
      .access_control_allow_methods("GET, POST")
      .frame_options(FrameOptions::SameOrigin)
      .content_security_policy("default-src 'self'")
      .cross_origin_opener_policy(CrossOriginOpenerPolicy::SameOrigin)
      .cross_origin_embedder_policy(CrossOriginEmbedderPolicy::RequireCorp)
      .build();

    assert_eq!(
      headers.get(headers::ACCESS_CONTROL_ALLOW_ORIGIN),
      Some(&"*".to_string())
    );
    assert_eq!(
      headers.get(headers::ACCESS_CONTROL_ALLOW_METHODS),
      Some(&"GET, POST".to_string())
    );
    assert_eq!(
      headers.get(headers::X_FRAME_OPTIONS),
      Some(&"SAMEORIGIN".to_string())
    );
    assert_eq!(
      headers.get(headers::CONTENT_SECURITY_POLICY),
      Some(&"default-src 'self'".to_string())
    );
    assert_eq!(
      headers.get(headers::CROSS_ORIGIN_OPENER_POLICY),
      Some(&"same-origin".to_string())
    );
    assert_eq!(
      headers.get(headers::CROSS_ORIGIN_EMBEDDER_POLICY),
      Some(&"require-corp".to_string())
    );
  }

  #[test]
  fn builds_an_empty_map_by_default() {
    let headers: HashMap<String, String> = CustomHeaders::builder().build();
    assert!(headers.is_empty());
  }

  #[test]
  fn supports_arbitrary_headers() {
    let headers = CustomHeaders::builder().header("X-Custom", "value").build();
    assert_eq!(headers.get("X-Custom"), Some(&"value".to_string()));
  }
}
