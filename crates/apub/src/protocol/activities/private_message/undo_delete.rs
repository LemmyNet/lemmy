use crate::{
  objects::person::ApubPerson,
  protocol::{activities::private_message::delete::DeletePrivateMessage, Unparsed},
};
use activitystreams_kinds::activity::UndoType;
use lemmy_apub_lib::object_id::ObjectId;
use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UndoDeletePrivateMessage {
  pub(crate) actor: ObjectId<ApubPerson>,
  pub(crate) to: [ObjectId<ApubPerson>; 1],
  pub(crate) object: DeletePrivateMessage,
  #[serde(rename = "type")]
  pub(crate) kind: UndoType,
  pub(crate) id: Url,
  #[serde(flatten)]
  pub(crate) unparsed: Unparsed,
}
