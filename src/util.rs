use std::{collections::HashMap, hash::Hash, ops::Deref};

pub struct Counter<T: Hash + Eq> {
  inner: HashMap<T, usize>,
}

impl<T: Hash + Eq> Counter<T> {
  pub fn new() -> Self {
    Self {
      inner: HashMap::new(),
    }
  }

  pub fn inc(&mut self, key: T) {
    let value = self.inner.entry(key).or_insert(0);
    *value += 1;
  }
}

impl<T: Hash + Eq> Deref for Counter<T> {
  type Target = HashMap<T, usize>;

  fn deref(&self) -> &Self::Target {
    &self.inner
  }
}

impl<T: Hash + Eq> Default for Counter<T> {
  fn default() -> Self {
    Self::new()
  }
}

#[cfg(test)]
pub mod tests {
  use super::*;

  #[test]
  fn test_counter() {
    let mut counter = Counter::new();
    counter.inc("abc");
    counter.inc("abc");
    let keys: Vec<&&str> = counter.keys().collect();
    assert_eq!(keys.len(), 1);
    assert_eq!(*keys[0], "abc");
    assert_eq!(counter.get("abc").unwrap(), &2);
  }
}
