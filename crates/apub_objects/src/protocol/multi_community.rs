use crate::objects::{
  community::ApubCommunity,
  multi_community::ApubMultiCommunity,
  multi_community_collection::ApubFeedCollection,
  person::ApubPerson,
};
use activitypub_federation::{
  fetch::{collection_id::CollectionId, object_id::ObjectId},
  kinds::collection::CollectionType,
  protocol::public_key::PublicKey,
};
use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Feed {
  pub r#type: FeedType,
  pub id: ObjectId<ApubMultiCommunity>,
  pub inbox: Url,
  pub public_key: PublicKey,

  pub following: CollectionId<ApubFeedCollection>,
  pub name: String,
  pub summary: Option<String>,
  pub content: Option<String>,
  pub attributed_to: ObjectId<ApubPerson>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq, Default)]
pub enum FeedType {
  #[default]
  Feed,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FeedCollection {
  pub r#type: CollectionType,
  pub id: CollectionId<ApubFeedCollection>,
  pub total_items: i32,
  pub items: Vec<ObjectId<ApubCommunity>>,
}
