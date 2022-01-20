use crate::{
  objects::{person::ApubPerson, private_message::ApubPrivateMessage},
  protocol::Unparsed,
};
use activitystreams_kinds::activity::DeleteType;
use lemmy_apub_lib::object_id::ObjectId;
use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DeletePrivateMessage {
  pub(crate) actor: ObjectId<ApubPerson>,
  #[serde(deserialize_with = "crate::deserialize_one")]
  pub(crate) to: [ObjectId<ApubPerson>; 1],
  pub(crate) object: ObjectId<ApubPrivateMessage>,
  #[serde(rename = "type")]
  pub(crate) kind: DeleteType,
  pub(crate) id: Url,
  #[serde(flatten)]
  pub(crate) unparsed: Unparsed,
}
