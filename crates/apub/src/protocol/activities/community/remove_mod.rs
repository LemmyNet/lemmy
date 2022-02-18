use crate::{objects::person::ApubPerson, protocol::Unparsed};
use activitystreams_kinds::activity::RemoveType;
use lemmy_apub_lib::object_id::ObjectId;
use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RemoveMod {
  pub(crate) actor: ObjectId<ApubPerson>,
  #[serde(deserialize_with = "crate::deserialize_one_or_many")]
  pub(crate) to: Vec<Url>,
  pub(crate) object: ObjectId<ApubPerson>,
  #[serde(deserialize_with = "crate::deserialize_one_or_many")]
  pub(crate) cc: Vec<Url>,
  #[serde(rename = "type")]
  pub(crate) kind: RemoveType,
  pub(crate) target: Url,
  pub(crate) id: Url,

  #[serde(flatten)]
  pub(crate) unparsed: Unparsed,
}
