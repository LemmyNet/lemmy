use crate::protocol::IdOrNestedObject;
use activitypub_federation::{
  fetch::object_id::ObjectId,
  kinds::{activity::AnnounceType, collection::OrderedCollectionType},
  protocol::helpers::deserialize_one_or_many,
};
use lemmy_apub_objects::objects::community::ApubCommunity;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use url::Url;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AnnounceActivity {
  pub(crate) actor: ObjectId<ApubCommunity>,
  #[serde(deserialize_with = "deserialize_one_or_many")]
  pub(crate) to: Vec<Url>,
  // #[serde(deserialize_with = "deserialize_one_or_many")]
  // pub object: Vec<IdOrNestedObject<RawAnnouncableActivities>>,
  // pub object: IdOrNestedObject<OneOrManyActivity>,
  pub object: IdOrNestedObject<OneOrManyActivity>,
  #[serde(deserialize_with = "deserialize_one_or_many")]
  pub(crate) cc: Vec<Url>,
  #[serde(rename = "type")]
  pub(crate) kind: AnnounceType,
  pub(crate) id: Url,
}

/// Use this to receive community inbox activities, and then announce them if valid. This
/// ensures that all json fields are kept, even if Lemmy doesn't understand them.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct RawAnnouncableActivities {
  pub(crate) id: Url,
  pub(crate) actor: Url,
  #[serde(flatten)]
  pub(crate) other: Map<String, Value>,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct AnnouncableActivitiesCollection {
  pub(crate) r#type: OrderedCollectionType,
  pub(crate) id: Url,
  pub(crate) total_items: i32,
  pub(crate) ordered_items: Vec<RawAnnouncableActivities>,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub enum OneOrManyActivity {
  One(RawAnnouncableActivities),
  Many(AnnouncableActivitiesCollection),
}
