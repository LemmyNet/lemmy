use crate::{
  objects::person::ApubPerson,
  protocol::{activities::CreateOrUpdateType, objects::private_message::PrivateMessage},
};
use activitypub_federation::{fetch::object_id::ObjectId, protocol::helpers::deserialize_one};
use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateOrUpdatePrivateMessage {
  pub(crate) id: Url,
  pub(crate) actor: ObjectId<ApubPerson>,
  #[serde(deserialize_with = "deserialize_one")]
  pub(crate) to: [ObjectId<ApubPerson>; 1],
  pub(crate) object: PrivateMessage,
  #[serde(rename = "type")]
  pub(crate) kind: CreateOrUpdateType,
}
