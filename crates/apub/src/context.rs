use activitystreams::{base::AnyBase, primitives::OneOrMany};
use serde::{Deserialize, Serialize};

lazy_static! {
  static ref CONTEXT: OneOrMany<AnyBase> =
    serde_json::from_str(include_str!("../assets/lemmy/context.json")).expect("parse context");
}

#[derive(Serialize, Deserialize)]
pub(crate) struct WithContext<T> {
  #[serde(rename = "@context")]
  context: OneOrMany<AnyBase>,
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
