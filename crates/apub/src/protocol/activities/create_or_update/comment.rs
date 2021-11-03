use crate::{
  fetcher::object_id::ObjectId,
  objects::person::ApubPerson,
  protocol::{activities::CreateOrUpdateType, objects::note::Note},
};
use activitystreams::{link::Mention, unparsed::Unparsed};
use lemmy_apub_lib::traits::ActivityFields;
use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Clone, Debug, Deserialize, Serialize, ActivityFields)]
#[serde(rename_all = "camelCase")]
pub struct CreateOrUpdateComment {
  pub(crate) actor: ObjectId<ApubPerson>,
  pub(crate) to: Vec<Url>,
  pub(crate) object: Note,
  pub(crate) cc: Vec<Url>,
  #[serde(default)]
  pub(crate) tag: Vec<Mention>,
  #[serde(rename = "type")]
  pub(crate) kind: CreateOrUpdateType,
  pub(crate) id: Url,
  #[serde(flatten)]
  pub(crate) unparsed: Unparsed,
}
