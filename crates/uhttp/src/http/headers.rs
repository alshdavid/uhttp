use std::sync::Arc;

use http::HeaderMap;
use http::HeaderName;
use http::HeaderValue;
use tokio::sync::RwLock;

use super::ResponseState;

pub struct Headers {
  state: Arc<RwLock<ResponseState>>,
}

impl Headers {
  pub(super) fn new(headers: Arc<RwLock<ResponseState>>) -> Self {
    Self { state: headers }
  }

  pub async fn inner_cloned(&self) -> crate::Result<HeaderMap> {
    todo!()
    // let guard = self.state.read().await;
    // match &*guard {
    //   ResponseState::Builder((builder, _)) => {
    //     if let Some(headers) = builder.headers_ref().cloned() {
    //       return Ok(headers);
    //     };
    //     return Err(crate::Error::generic("No headers present"));
    //   }
    //   ResponseState::Stream(_) => {
    //     return Err(crate::Error::generic("Headers already sent"));
    //   }
    //   ResponseState::Done => {
    //     return Err(crate::Error::generic("Request already sent"));
    //   }
    //   ResponseState::Pending => {
    //     return Err(crate::Error::generic("Request currently sending"));
    //   }
    // }
  }

  pub async fn replace(
    &mut self,
    headers: HeaderMap,
  ) -> crate::Result<()> {
    todo!()
    // let mut guard = self.state.write().await;
    // match &mut *guard {
    //   ResponseState::Builder((builder, _)) => {
    //     if let Some(current) = builder.headers_mut() {
    //       *current = headers;
    //     }
    //     Ok(())
    //   }
    //   ResponseState::Stream(_) => {
    //     return Err(crate::Error::generic("Headers already sent"));
    //   }
    //   ResponseState::Done => {
    //     return Err(crate::Error::generic("Request already sent"));
    //   }
    //   ResponseState::Pending => {
    //     return Err(crate::Error::generic("Request currently sending"));
    //   }
    // }
  }

  pub async fn add(
    &mut self,
    key: &str,
    value: &str,
  ) -> crate::Result<bool> {
    todo!()
    // let mut guard = self.state.write().await;
    // match &mut *guard {
    //   ResponseState::Builder((builder, _)) => {
    //     let Some(headers) = builder.headers_mut() else {
    //       return Err(crate::Error::generic("No headers"));
    //     };
    //     let Ok(header) = HeaderValue::from_str(value) else {
    //       return Err(crate::Error::generic("Invalid header value"));
    //     };
    //     let Ok(key) = HeaderName::from_bytes(key.as_bytes()) else {
    //       return Err(crate::Error::generic("Invalid header key"));
    //     };
    //     return Ok(headers.append(key, header));
    //   }
    //   ResponseState::Stream(_) => {
    //     return Err(crate::Error::generic("Headers already sent"));
    //   }
    //   ResponseState::Done => {
    //     return Err(crate::Error::generic("Request already sent"));
    //   }
    //   ResponseState::Pending => {
    //     return Err(crate::Error::generic("Request currently sending"));
    //   }
    // };
  }

  pub async fn set(
    &mut self,
    key: &str,
    value: &str,
  ) -> crate::Result<Option<String>> {
    todo!()
    // let mut guard = self.state.write().await;
    // match &mut *guard {
    //   ResponseState::Builder((builder, _)) => {
    //     let Some(headers) = builder.headers_mut() else {
    //       return Err(crate::Error::generic("No headers"));
    //     };
    //     let Ok(header) = HeaderValue::from_str(value) else {
    //       return Err(crate::Error::generic("Invalid header value"));
    //     };
    //     let Ok(key) = HeaderName::from_bytes(key.as_bytes()) else {
    //       return Err(crate::Error::generic("Invalid header key"));
    //     };
    //     if let Some(prev) = headers.insert(key, header) {
    //       if let Ok(prev) = prev.to_str() {
    //         return Ok(Some(prev.to_string()));
    //       }
    //       return Err(crate::Error::generic("Unable to parse existing header"));
    //     };
    //     return Ok(None);
    //   }
    //   ResponseState::Stream(_) => {
    //     return Err(crate::Error::generic("Headers already sent"));
    //   }
    //   ResponseState::Done => {
    //     return Err(crate::Error::generic("Request already sent"));
    //   }
    //   ResponseState::Pending => {
    //     return Err(crate::Error::generic("Request currently sending"));
    //   }
    // };
  }

  pub async fn get(
    &self,
    key: &str,
  ) -> Option<String> {
    todo!()
    // let guard = self.state.read().await;
    // match &*guard {
    //   ResponseState::Builder((builder, _)) => {
    //     let Some(headers) = builder.headers_ref() else {
    //       return None;
    //     };
    //     headers
    //       .get(key)
    //       .and_then(|v| v.to_str().ok())
    //       .map(|s| s.to_string())
    //   }
    //   _ => None,
    // }
  }

  pub async fn get_all(
    &self,
    key: &str,
  ) -> Vec<String> {
    todo!()
    // let guard = self.state.read().await;
    // match &*guard {
    //   ResponseState::Builder((builder, _)) => {
    //     let Some(headers) = builder.headers_ref() else {
    //       return Vec::new();
    //     };
    //     headers
    //       .get_all(key)
    //       .into_iter()
    //       .filter_map(|v| v.to_str().ok())
    //       .map(|s| s.to_string())
    //       .collect()
    //   }
    //   _ => Vec::new(),
    // }
  }
}
