use activitypub_federation::{fetch::object_id::ObjectId, kinds::collection::CollectionType};
use lemmy_apub_objects::objects::community::ApubCommunity;
use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MultiCommunityCollection {
  pub(crate) r#type: CollectionType,
  pub(crate) id: Url,
  pub(crate) total_items: i32,
  pub(crate) items: Vec<ObjectId<ApubCommunity>>,
}
