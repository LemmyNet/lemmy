use crate::protocol::activities::create_or_update::post::CreateOrUpdatePost;
use activitystreams::collection::kind::OrderedCollectionType;
use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GroupOutbox {
  pub(crate) r#type: OrderedCollectionType,
  pub(crate) id: Url,
  pub(crate) total_items: i32,
  pub(crate) ordered_items: Vec<CreateOrUpdatePost>,
}
