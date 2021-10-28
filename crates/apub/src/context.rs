use activitystreams::{base::AnyBase, context, primitives::OneOrMany};
use serde::{Deserialize, Serialize};
use serde_json::json;
use url::Url;

lazy_static! {
  static ref CONTEXT: OneOrMany<AnyBase> = {
    let context_ext = AnyBase::from_arbitrary_json(json!(
    {
      "sc": "http://schema.org#",
      "sensitive": "as:sensitive",
      "stickied": "as:stickied",
      "pt": "https://join-lemmy.org#",
      "comments_enabled": {
        "type": "sc:Boolean",
        "id": "pt:commentsEnabled"
      },
      "moderators": "as:moderators",
      "matrixUserId": {
        "type": "sc:Text",
        "id": "as:alsoKnownAs"
      },
    }))
    .expect("parse context");
    OneOrMany::from(vec![
      AnyBase::from(context()),
      context_ext,
      AnyBase::from(Url::parse("https://w3id.org/security/v1").expect("parse context")),
    ])
  };
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
