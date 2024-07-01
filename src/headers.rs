use std::collections::HashMap;

pub type RawHeaders = HashMap<String, String>;
pub type ParsedHeaders = HashMap<String, Vec<String>>;

#[derive(Default, Debug)]
pub struct Headers {
  pub(super) internal: HashMap<String, Vec<String>>,
}

impl Headers {
  pub fn from_raw(raw_headers: RawHeaders) -> Self {
    let mut internal = ParsedHeaders::new();

    for (key, value) in raw_headers {
      internal.insert(key, value.split(",").map(|s| s.to_string()).collect());
    }

    Self {
      internal,
    }
  }

  pub fn add<K: AsRef<str>, V: AsRef<str>>(
    &mut self,
    key: K,
    value: V,
  ) {
    if let Some(values) = self.internal.get_mut(key.as_ref()) {
      values.push(value.as_ref().to_string());
    } else {
      self
        .internal
        .insert(key.as_ref().to_string(), vec![value.as_ref().to_string()]);
    }
  }

  pub fn replace<K: AsRef<str>, V: AsRef<str>>(
    &mut self,
    key: K,
    values: &[V],
  ) {
    self.internal.insert(
      key.as_ref().to_string(),
      values
        .iter()
        .map(|v| v.as_ref().to_string())
        .collect::<Vec<String>>(),
    );
  }

  pub fn remove<K: AsRef<str>>(
    &mut self,
    key: K,
  ) -> Option<Vec<String>> {
    self.internal.remove(key.as_ref())
  }

  pub fn get<'a, K: AsRef<str>>(
    &'a self,
    key: K,
  ) -> Option<&'a Vec<String>> {
    self.internal.get(key.as_ref())
  }

  pub fn iter<'a>(&'a mut self) -> Box<dyn Iterator<Item = (&'a String, &'a Vec<String>)> + 'a> {
    Box::new(self.internal.iter())
  }

  pub fn iter_mut<'a>(
    &'a mut self
  ) -> Box<dyn Iterator<Item = (&'a String, &'a mut Vec<String>)> + 'a> {
    Box::new(self.internal.iter_mut())
  }
}
