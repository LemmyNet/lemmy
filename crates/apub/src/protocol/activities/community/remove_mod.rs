use crate::objects::person::ApubPerson;
use activitystreams::{activity::kind::RemoveType, primitives::OneOrMany, unparsed::Unparsed};
use lemmy_apub_lib::object_id::ObjectId;
use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RemoveMod {
  pub(crate) actor: ObjectId<ApubPerson>,
  pub(crate) to: Option<OneOrMany<Url>>,
  pub(crate) object: ObjectId<ApubPerson>,
  pub(crate) cc: Option<OneOrMany<Url>>,
  #[serde(rename = "type")]
  pub(crate) kind: RemoveType,
  pub(crate) target: Url,
  pub(crate) id: Url,
  #[serde(flatten)]
  pub(crate) unparsed: Unparsed,
}
