use crate::{objects::person::ApubPerson, protocol::activities::following::follow::Follow};
use activitypub_federation::{
  fetch::object_id::ObjectId, kinds::activity::UndoType, protocol::helpers::deserialize_skip_error,
};
use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UndoFollow {
  pub(crate) actor: ObjectId<ApubPerson>,
  /// Optional, for compatibility with platforms that always expect recipient field
  #[serde(deserialize_with = "deserialize_skip_error", default)]
  pub(crate) to: Option<[ObjectId<ApubPerson>; 1]>,
  pub(crate) object: Follow,
  #[serde(rename = "type")]
  pub(crate) kind: UndoType,
  pub(crate) id: Url,
}
