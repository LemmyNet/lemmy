use crate::objects::person::ApubPerson;
use activitystreams_kinds::activity::DeleteType;
use lemmy_apub_lib::object_id::ObjectId;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use url::Url;

#[skip_serializing_none]
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DeleteUser {
  pub(crate) actor: ObjectId<ApubPerson>,
  #[serde(deserialize_with = "crate::deserialize_one_or_many")]
  pub(crate) to: Vec<Url>,
  pub(crate) object: ObjectId<ApubPerson>,
  #[serde(rename = "type")]
  pub(crate) kind: DeleteType,
  pub(crate) id: Url,

  #[serde(deserialize_with = "crate::deserialize_one_or_many", default)]
  #[serde(skip_serializing_if = "Vec::is_empty")]
  pub(crate) cc: Vec<Url>,
}
