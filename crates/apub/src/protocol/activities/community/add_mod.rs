use crate::{objects::person::ApubPerson, protocol::Unparsed};
use activitystreams_kinds::activity::AddType;
use lemmy_apub_lib::object_id::ObjectId;
use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AddMod {
  pub(crate) actor: ObjectId<ApubPerson>,
  pub(crate) to: Vec<Url>,
  pub(crate) object: ObjectId<ApubPerson>,
  pub(crate) target: Url,
  pub(crate) cc: Vec<Url>,
  #[serde(rename = "type")]
  pub(crate) kind: AddType,
  pub(crate) id: Url,
  #[serde(flatten)]
  pub(crate) unparsed: Unparsed,
}
