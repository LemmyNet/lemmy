use crate::{
  objects::{
    community::ApubCommunity,
    multi_community::ApubMultiCommunity,
    multi_community_collection::ApubFeedCollection,
    person::ApubPerson,
  },
  utils::protocol::Source,
};
use activitypub_federation::{
  fetch::{collection_id::CollectionId, object_id::ObjectId},
  kinds::collection::CollectionType,
  protocol::{helpers::deserialize_skip_error, public_key::PublicKey, values::MediaTypeHtml},
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
  /// username, set at account creation and usually fixed after that
  pub preferred_username: String,
  /// title
  pub name: Option<String>,
  /// short instance description
  pub(crate) content: Option<String>,
  /// sidebar
  #[serde(deserialize_with = "deserialize_skip_error", default)]
  pub source: Option<Source>,
  pub(crate) media_type: Option<MediaTypeHtml>,
  // sidebar
  pub summary: Option<String>,
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
