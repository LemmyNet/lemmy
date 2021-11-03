use crate::{
  fetcher::object_id::ObjectId,
  objects::{person::ApubPerson, private_message::ApubPrivateMessage},
};
use activitystreams::{activity::kind::DeleteType, unparsed::Unparsed};
use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DeletePrivateMessage {
  pub(crate) actor: ObjectId<ApubPerson>,
  pub(crate) to: [ObjectId<ApubPerson>; 1],
  pub(crate) object: ObjectId<ApubPrivateMessage>,
  #[serde(rename = "type")]
  pub(crate) kind: DeleteType,
  pub(crate) id: Url,
  #[serde(flatten)]
  pub(crate) unparsed: Unparsed,
}
