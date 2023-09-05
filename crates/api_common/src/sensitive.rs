use serde::{Deserialize, Serialize};
use std::{
    borrow::Borrow,
    ops::{Deref, DerefMut},
};
#[cfg(feature = "full")]
use ts_rs::TS;

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Deserialize, Serialize, Default)]
#[serde(transparent)]
pub struct Sensitive<T>(T);

impl<T> Sensitive<T> {
    pub fn new(item: T) -> Self {
        Sensitive(item)
    }
    pub fn into_inner(self) -> T {
        self.0
    }
}

impl<T> std::fmt::Debug for Sensitive<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Sensitive").finish()
    }
}

impl<T> AsRef<T> for Sensitive<T> {
    fn as_ref(&self) -> &T {
        &self.0
    }
}

impl AsRef<str> for Sensitive<String> {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl AsRef<[u8]> for Sensitive<String> {
    fn as_ref(&self) -> &[u8] {
        self.0.as_ref()
    }
}

impl AsRef<[u8]> for Sensitive<Vec<u8>> {
    fn as_ref(&self) -> &[u8] {
        self.0.as_ref()
    }
}

impl<T> AsMut<T> for Sensitive<T> {
    fn as_mut(&mut self) -> &mut T {
        &mut self.0
    }
}

impl AsMut<str> for Sensitive<String> {
    fn as_mut(&mut self) -> &mut str {
        &mut self.0
    }
}

impl Deref for Sensitive<String> {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Sensitive<String> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<T> From<T> for Sensitive<T> {
    fn from(t: T) -> Self {
        Sensitive(t)
    }
}

impl From<&str> for Sensitive<String> {
    fn from(s: &str) -> Self {
        Sensitive(s.into())
    }
}

impl<T> Borrow<T> for Sensitive<T> {
    fn borrow(&self) -> &T {
        &self.0
    }
}

impl Borrow<str> for Sensitive<String> {
    fn borrow(&self) -> &str {
        &self.0
    }
}

#[cfg(feature = "full")]
impl TS for Sensitive<String> {
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
