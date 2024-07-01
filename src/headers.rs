use std::{collections::HashMap, io};

pub type RawHeaders = HashMap<String, String>;
pub type ParsedHeaders = HashMap<String, Vec<String>>;

pub(super) type HeaderRaw = (
  Vec<u8>,
  Vec<u8>,
);

#[derive(Default, Debug)]
pub struct Headers {
  pub(super) internal: HashMap<String, String>,
}

impl Headers {
  pub fn add<K: AsRef<str>, V: AsRef<str>>(
    &mut self,
    key: K,
    value: V,
  ) {
    if let Some(values) = self.internal.get_mut(key.as_ref()) {
      values.push_str(", ");
      values.push_str(value.as_ref());
    } else {
      self
        .internal
        .insert(key.as_ref().to_string(), value.as_ref().to_string());
    }
  }

  pub fn replace<K: AsRef<str>, V: AsRef<str>>(
    &mut self,
    key: K,
    value: V,
  ) {
    self.internal.insert(
      key.as_ref().to_string(),
      value.as_ref().to_string(),
    );
  }

  pub fn remove<K: AsRef<str>>(
    &mut self,
    key: K,
  ) -> Option<String> {
    self.internal.remove(key.as_ref())
  }

  pub fn get<'a, K: AsRef<str>>(
    &'a self,
    key: K,
  ) -> Option<&'a String> {
    self.internal.get(key.as_ref())
  }

  pub fn iter<'a>(&'a mut self) -> Box<dyn Iterator<Item = (&'a String, &'a String)> + 'a> {
    Box::new(self.internal.iter())
  }

  pub fn iter_mut<'a>(
    &'a mut self
  ) -> Box<dyn Iterator<Item = (&'a String, &'a mut String)> + 'a> {
    Box::new(self.internal.iter_mut())
  }
}
