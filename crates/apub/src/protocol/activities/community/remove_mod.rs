use crate::{fetcher::object_id::ObjectId, objects::person::ApubPerson};
use activitystreams::{activity::kind::RemoveType, unparsed::Unparsed};
use lemmy_apub_lib::traits::ActivityFields;
use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Clone, Debug, Deserialize, Serialize, ActivityFields)]
#[serde(rename_all = "camelCase")]
pub struct RemoveMod {
  pub(crate) actor: ObjectId<ApubPerson>,
  pub(crate) to: Vec<Url>,
  pub(crate) object: ObjectId<ApubPerson>,
  pub(crate) cc: Vec<Url>,
  #[serde(rename = "type")]
  pub(crate) kind: RemoveType,
  pub(crate) target: Url,
  pub(crate) id: Url,
  #[serde(flatten)]
  pub(crate) unparsed: Unparsed,
}
