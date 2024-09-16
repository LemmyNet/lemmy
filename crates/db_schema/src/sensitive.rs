use serde::{Deserialize, Serialize};
use std::{fmt::Debug, ops::Deref};
#[cfg(feature = "full")]
use ts_rs::TS;

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Deserialize, Serialize, Default)]
#[cfg_attr(feature = "full", derive(DieselNewType))]
#[serde(transparent)]
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

#[cfg(feature = "full")]
impl TS for SensitiveString {
  fn name() -> String {
    "string".to_string()
  }
  fn name_with_type_args(_args: Vec<String>) -> String {
    "string".to_string()
  }
  fn dependencies() -> Vec<ts_rs::Dependency> {
    Vec::new()
  }
  fn transparent() -> bool {
    true
  }
}
