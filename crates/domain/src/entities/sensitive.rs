use std::{fmt::Debug, ops::Deref};

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default, DieselNewType)]
pub struct SensitiveString(String);

impl SensitiveString {
  pub fn into_inner(self) -> String {
    self.0
  }
}

impl Debug for SensitiveString {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("Sensitive").finish()
  }
}

impl AsRef<[u8]> for SensitiveString {
  fn as_ref(&self) -> &[u8] {
    self.0.as_ref()
  }
}

impl Deref for SensitiveString {
  type Target = str;

  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

impl From<String> for SensitiveString {
  fn from(t: String) -> Self {
    SensitiveString(t)
  }
}
