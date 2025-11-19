use super::block_user::BlockUser;
use activitypub_federation::{
  fetch::object_id::ObjectId,
  kinds::activity::UndoType,
  protocol::helpers::deserialize_one_or_many,
};
use lemmy_apub_objects::objects::{community::ApubCommunity, person::ApubPerson};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use url::Url;

#[skip_serializing_none]
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UndoBlockUser {
  pub(crate) actor: ObjectId<ApubPerson>,
  #[serde(deserialize_with = "deserialize_one_or_many")]
  pub(crate) to: Vec<Url>,
  pub(crate) object: BlockUser,
  #[serde(deserialize_with = "deserialize_one_or_many")]
  pub(crate) cc: Vec<Url>,
  #[serde(rename = "type")]
  pub(crate) kind: UndoType,
  pub(crate) id: Url,
  pub(crate) audience: Option<ObjectId<ApubCommunity>>,

  /// Quick and dirty solution.
  /// TODO: send a separate Delete activity instead
  pub(crate) restore_data: Option<bool>,
}
