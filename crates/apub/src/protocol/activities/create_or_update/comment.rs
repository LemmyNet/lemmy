use crate::{
  objects::person::ApubPerson,
  protocol::{activities::CreateOrUpdateType, objects::note::Note},
};
use activitystreams::{link::Mention, primitives::OneOrMany, unparsed::Unparsed};
use lemmy_apub_lib::object_id::ObjectId;
use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateOrUpdateComment {
  pub(crate) actor: ObjectId<ApubPerson>,
  pub(crate) to: Option<OneOrMany<Url>>,
  pub(crate) object: Note,
  pub(crate) cc: Option<OneOrMany<Url>>,
  #[serde(default)]
  pub(crate) tag: Vec<Mention>,
  #[serde(rename = "type")]
  pub(crate) kind: CreateOrUpdateType,
  pub(crate) id: Url,
  #[serde(flatten)]
  pub(crate) unparsed: Unparsed,
}
