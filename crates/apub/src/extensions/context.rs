use activitystreams::{base::AnyBase, context};
use lemmy_utils::LemmyError;
use serde_json::json;

pub(crate) fn lemmy_context() -> Result<Vec<AnyBase>, LemmyError> {
  let context_ext = AnyBase::from_arbitrary_json(json!(
  {
    "sc": "http://schema.org#",
    "category": "sc:category",
    "sensitive": "as:sensitive",
    "stickied": "as:stickied",
    "comments_enabled": {
    "kind": "sc:Boolean",
    "id": "pt:commentsEnabled"
    }
  }))?;
  Ok(vec![AnyBase::from(context()), context_ext])
}
