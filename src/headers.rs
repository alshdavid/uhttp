use std::collections::HashMap;

pub type RawHeaders = HashMap<String, String>;
pub type ParsedHeaders = HashMap<String, Vec<String>>;

#[derive(Default)]
pub struct Headers {
  internal: HashMap<String, String>,
}

impl std::fmt::Debug for Headers {
  fn fmt(
    &self,
    f: &mut std::fmt::Formatter<'_>,
  ) -> std::fmt::Result {
    f.debug_map().entries(&self.internal).finish()
  }
}

impl Headers {
  pub fn set<K: AsRef<str>, V: AsRef<str>>(
    &mut self,
    key: K,
    value: V,
  ) {
    self
      .internal
      .insert(key.as_ref().to_string(), value.as_ref().to_string());
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

  pub fn iter_mut<'a>(&'a mut self) -> Box<dyn Iterator<Item = (&'a String, &'a mut String)> + 'a> {
    Box::new(self.internal.iter_mut())
  }
}
