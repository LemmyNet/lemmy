use std::{
  borrow::Borrow,
  ops::{Deref, DerefMut},
};

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Deserialize, serde::Serialize)]
#[serde(transparent)]
pub struct Sensitive(String);

impl Sensitive {
  pub fn new(string: String) -> Self {
    Sensitive(string)
  }

  pub fn into_inner(this: Self) -> String {
    this.0
  }
}

impl std::fmt::Debug for Sensitive {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("Sensitive").finish()
  }
}

impl AsRef<String> for Sensitive {
  fn as_ref(&self) -> &String {
    &self.0
  }
}

impl AsRef<str> for Sensitive {
  fn as_ref(&self) -> &str {
    &self.0
  }
}

impl AsRef<[u8]> for Sensitive {
  fn as_ref(&self) -> &[u8] {
    self.0.as_ref()
  }
}

impl AsMut<String> for Sensitive {
  fn as_mut(&mut self) -> &mut String {
    &mut self.0
  }
}

impl AsMut<str> for Sensitive {
  fn as_mut(&mut self) -> &mut str {
    &mut self.0
  }
}

impl Deref for Sensitive {
  type Target = str;

  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

impl DerefMut for Sensitive {
  fn deref_mut(&mut self) -> &mut Self::Target {
    &mut self.0
  }
}

impl From<String> for Sensitive {
  fn from(s: String) -> Self {
    Sensitive(s)
  }
}

impl From<&str> for Sensitive {
  fn from(s: &str) -> Self {
    Sensitive(s.into())
  }
}

impl Borrow<String> for Sensitive {
  fn borrow(&self) -> &String {
    &self.0
  }
}

impl Borrow<str> for Sensitive {
  fn borrow(&self) -> &str {
    &self.0
  }
}
