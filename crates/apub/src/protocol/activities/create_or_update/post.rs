use crate::{
  objects::person::ApubPerson,
  protocol::{activities::CreateOrUpdateType, objects::page::Page, Unparsed},
};
use lemmy_apub_lib::{deser::deserialize_one_or_many, object_id::ObjectId};
use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateOrUpdatePost {
  pub(crate) actor: ObjectId<ApubPerson>,
  #[serde(deserialize_with = "deserialize_one_or_many")]
  pub(crate) to: Vec<Url>,
  pub(crate) object: Page,
  #[serde(deserialize_with = "deserialize_one_or_many")]
  pub(crate) cc: Vec<Url>,
  #[serde(rename = "type")]
  pub(crate) kind: CreateOrUpdateType,
  pub(crate) id: Url,

  #[serde(flatten)]
  pub(crate) unparsed: Unparsed,
}
