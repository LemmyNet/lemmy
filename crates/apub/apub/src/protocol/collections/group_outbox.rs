use activitypub_federation::kinds::collection::{OrderedCollectionPageType, OrderedCollectionType};
use lemmy_apub_activities::protocol::community::announce::AnnounceActivity;
use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GroupOutbox {
  pub(crate) r#type: OrderedCollectionType,
  pub(crate) id: Url,
  pub(crate) total_items: i32,
  /// Next collection page. Present when locally generated outboxes have more items than
  /// fit in `ordered_items`. Optional so older remote collections still parse.
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub(crate) next: Option<Url>,
  pub(crate) ordered_items: Vec<AnnounceActivity>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GroupOutboxPage {
  pub(crate) r#type: OrderedCollectionPageType,
  pub(crate) id: Url,
  pub(crate) part_of: Url,
  pub(crate) total_items: i32,
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub(crate) next: Option<Url>,
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub(crate) prev: Option<Url>,
  pub(crate) ordered_items: Vec<AnnounceActivity>,
}
