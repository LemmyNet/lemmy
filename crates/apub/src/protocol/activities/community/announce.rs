use crate::{objects::community::ApubCommunity, protocol::IdOrNestedObject};
use activitypub_federation::{
  fetch::object_id::ObjectId,
  kinds::activity::AnnounceType,
  protocol::helpers::deserialize_one_or_many,
};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use url::Url;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AnnounceActivity {
  pub(crate) actor: ObjectId<ApubCommunity>,
  #[serde(deserialize_with = "deserialize_one_or_many")]
  pub(crate) to: Vec<Url>,
  pub(crate) object: IdOrNestedObject<RawAnnouncableActivities>,
  #[serde(deserialize_with = "deserialize_one_or_many")]
  pub(crate) cc: Vec<Url>,
  #[serde(rename = "type")]
  pub(crate) kind: AnnounceType,
  pub(crate) id: Url,
}

/// Use this to receive community inbox activities, and then announce them if valid. This
/// ensures that all json fields are kept, even if Lemmy doesnt understand them.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct RawAnnouncableActivities {
  pub(crate) id: Url,
  pub(crate) actor: Url,
  #[serde(flatten)]
  pub(crate) other: Map<String, Value>,
}
