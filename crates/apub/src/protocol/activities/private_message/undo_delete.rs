use crate::{
  fetcher::object_id::ObjectId,
  objects::person::ApubPerson,
  protocol::activities::private_message::delete::DeletePrivateMessage,
};
use activitystreams::{activity::kind::UndoType, unparsed::Unparsed};
use lemmy_apub_lib::traits::ActivityFields;
use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Clone, Debug, Deserialize, Serialize, ActivityFields)]
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
