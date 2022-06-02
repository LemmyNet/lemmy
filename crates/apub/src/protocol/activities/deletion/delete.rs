use crate::{
  objects::person::ApubPerson,
  protocol::{objects::tombstone::Tombstone, IdOrNestedObject, Unparsed},
};
use activitypub_federation::{core::object_id::ObjectId, deser::helpers::deserialize_one_or_many};
use activitystreams_kinds::activity::DeleteType;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use url::Url;

#[skip_serializing_none]
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Delete {
  pub(crate) actor: ObjectId<ApubPerson>,
  #[serde(deserialize_with = "deserialize_one_or_many")]
  pub(crate) to: Vec<Url>,
  pub(crate) object: IdOrNestedObject<Tombstone>,
  #[serde(rename = "type")]
  pub(crate) kind: DeleteType,
  pub(crate) id: Url,

  #[serde(deserialize_with = "deserialize_one_or_many")]
  #[serde(default)]
  #[serde(skip_serializing_if = "Vec::is_empty")]
  pub(crate) cc: Vec<Url>,
  /// If summary is present, this is a mod action (Remove in Lemmy terms). Otherwise, its a user
  /// deleting their own content.
  pub(crate) summary: Option<String>,
  #[serde(flatten)]
  pub(crate) unparsed: Unparsed,
}
