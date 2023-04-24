use crate::{objects::person::ApubPerson, protocol::activities::following::follow::Follow};
use activitypub_federation::core::object_id::ObjectId;
use activitystreams_kinds::activity::UndoType;
use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UndoFollow {
  pub(crate) actor: ObjectId<ApubPerson>,
  /// Optional, for compatibility with platforms that always expect recipient field
  pub(crate) to: Option<[ObjectId<ApubPerson>; 1]>,
  pub(crate) object: Follow,
  #[serde(rename = "type")]
  pub(crate) kind: UndoType,
  pub(crate) id: Url,
}
