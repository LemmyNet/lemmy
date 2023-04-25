use crate::{
  objects::{community::ApubCommunity, person::ApubPerson},
  protocol::activities::following::follow::Follow,
};
use activitypub_federation::{fetch::object_id::ObjectId, kinds::activity::AcceptType};
use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AcceptFollow {
  pub(crate) actor: ObjectId<ApubCommunity>,
  /// Optional, for compatibility with platforms that always expect recipient field
  pub(crate) to: Option<[ObjectId<ApubPerson>; 1]>,
  pub(crate) object: Follow,
  #[serde(rename = "type")]
  pub(crate) kind: AcceptType,
  pub(crate) id: Url,
}
