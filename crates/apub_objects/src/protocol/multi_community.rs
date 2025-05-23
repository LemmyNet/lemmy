use crate::objects::{community::ApubCommunity, person::ApubPerson};
use activitypub_federation::{fetch::object_id::ObjectId, kinds::collection::CollectionType};
use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MultiCommunityCollection {
  pub r#type: CollectionType,
  pub id: Url,
  pub total_items: i32,
  pub items: Vec<ObjectId<ApubCommunity>>,
  pub name: String,
  pub summary: Option<String>,
  pub content: Option<String>,
  pub attributed_to: ObjectId<ApubPerson>,
}
