use activitypub_federation::kinds::collection::OrderedCollectionType;
use lemmy_utils::error::LemmyError;
use serde::{Deserialize, Serialize};
use url::Url;

/// Empty placeholder outbox used for Person, Instance, which dont implement a proper outbox yet.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct EmptyOutbox {
  r#type: OrderedCollectionType,
  id: Url,
  ordered_items: Vec<()>,
  total_items: i32,
}

impl EmptyOutbox {
  pub(crate) fn new(outbox_id: Url) -> Result<EmptyOutbox, LemmyError> {
    Ok(EmptyOutbox {
      r#type: OrderedCollectionType::OrderedCollection,
      id: outbox_id,
      ordered_items: vec![],
      total_items: 0,
    })
  }
}
