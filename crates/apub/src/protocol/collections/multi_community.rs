use activitypub_federation::{fetch::object_id::ObjectId, kinds::collection::CollectionType};
use lemmy_apub_objects::objects::{community::ApubCommunity, person::ApubPerson};
use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MultiCommunityCollection {
  pub(crate) r#type: CollectionType,
  pub(crate) id: Url,
  pub(crate) total_items: i32,
  pub(crate) items: Vec<ObjectId<ApubCommunity>>,
  pub(crate) name: String,
  pub(crate) summary: Option<String>,
  pub(crate) content: Option<String>,
  pub(crate) attributed_to: ObjectId<ApubPerson>,
}
