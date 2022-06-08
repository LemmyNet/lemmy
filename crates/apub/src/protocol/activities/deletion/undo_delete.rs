use crate::{
  objects::person::ApubPerson,
  protocol::{activities::deletion::delete::Delete, Unparsed},
};
use activitypub_federation::{core::object_id::ObjectId, deser::helpers::deserialize_one_or_many};
use activitystreams_kinds::activity::UndoType;
use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UndoDelete {
  pub(crate) actor: ObjectId<ApubPerson>,
  #[serde(deserialize_with = "deserialize_one_or_many")]
  pub(crate) to: Vec<Url>,
  pub(crate) object: Delete,
  #[serde(rename = "type")]
  pub(crate) kind: UndoType,
  pub(crate) id: Url,

  #[serde(deserialize_with = "deserialize_one_or_many", default)]
  #[serde(skip_serializing_if = "Vec::is_empty")]
  pub(crate) cc: Vec<Url>,
  #[serde(flatten)]
  pub(crate) unparsed: Unparsed,
}
