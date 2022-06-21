use crate::{
  activity_lists::AnnouncableActivities,
  objects::community::ApubCommunity,
  protocol::{IdOrNestedObject, Unparsed},
};
use activitypub_federation::{core::object_id::ObjectId, deser::helpers::deserialize_one_or_many};
use activitystreams_kinds::activity::AnnounceType;
use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AnnounceActivity {
  pub(crate) actor: ObjectId<ApubCommunity>,
  #[serde(deserialize_with = "deserialize_one_or_many")]
  pub(crate) to: Vec<Url>,
  pub(crate) object: IdOrNestedObject<AnnouncableActivities>,
  #[serde(deserialize_with = "deserialize_one_or_many")]
  pub(crate) cc: Vec<Url>,
  #[serde(rename = "type")]
  pub(crate) kind: AnnounceType,
  pub(crate) id: Url,

  #[serde(flatten)]
  pub(crate) unparsed: Unparsed,
}
