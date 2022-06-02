use crate::{
  objects::person::ApubPerson,
  protocol::{objects::group::Group, Unparsed},
};
use activitypub_federation::{core::object_id::ObjectId, deser::helpers::deserialize_one_or_many};
use activitystreams_kinds::activity::UpdateType;
use serde::{Deserialize, Serialize};
use url::Url;

/// This activity is received from a remote community mod, and updates the description or other
/// fields of a local community.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateCommunity {
  pub(crate) actor: ObjectId<ApubPerson>,
  #[serde(deserialize_with = "deserialize_one_or_many")]
  pub(crate) to: Vec<Url>,
  // TODO: would be nice to use a separate struct here, which only contains the fields updated here
  pub(crate) object: Box<Group>,
  #[serde(deserialize_with = "deserialize_one_or_many")]
  pub(crate) cc: Vec<Url>,
  #[serde(rename = "type")]
  pub(crate) kind: UpdateType,
  pub(crate) id: Url,

  #[serde(flatten)]
  pub(crate) unparsed: Unparsed,
}
