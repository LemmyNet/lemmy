use serde::{Deserialize, Serialize};

lazy_static! {
  static ref CONTEXT: Vec<serde_json::Value> =
    serde_json::from_str(include_str!("../assets/lemmy/context.json")).expect("parse context");
}

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct WithContext<T> {
  #[serde(rename = "@context")]
  context: Vec<serde_json::Value>,
  #[serde(flatten)]
  inner: T,
}

impl<T> WithContext<T> {
  pub(crate) fn new(inner: T) -> WithContext<T> {
    WithContext {
      context: CONTEXT.clone(),
      inner,
    }
  }
  pub(crate) fn inner(self) -> T {
    self.inner
  }
}
