use activitypub_federation::kinds::collection::OrderedCollectionType;
use lemmy_apub_activities::protocol::community::announce::AnnounceActivity;
use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GroupOutbox {
  pub(crate) r#type: OrderedCollectionType,
  pub(crate) id: Url,
  pub(crate) total_items: i32,
  pub(crate) ordered_items: Vec<AnnounceActivity>,
}
