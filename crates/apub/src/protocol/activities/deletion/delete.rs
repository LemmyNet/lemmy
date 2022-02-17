use crate::{objects::person::ApubPerson, protocol::Unparsed};
use activitystreams_kinds::activity::DeleteType;
use lemmy_apub_lib::object_id::ObjectId;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use url::Url;

#[skip_serializing_none]
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Delete {
  pub(crate) actor: ObjectId<ApubPerson>,
  #[serde(deserialize_with = "crate::deserialize_one_or_many")]
  pub(crate) to: Vec<Url>,
  pub(crate) object: IdOrNestedObject,
  #[serde(rename = "type")]
  pub(crate) kind: DeleteType,
  pub(crate) id: Url,

  #[serde(deserialize_with = "crate::deserialize_one_or_many")]
  #[serde(default)]
  #[serde(skip_serializing_if = "Vec::is_empty")]
  pub(crate) cc: Vec<Url>,
  /// If summary is present, this is a mod action (Remove in Lemmy terms). Otherwise, its a user
  /// deleting their own content.
  pub(crate) summary: Option<String>,
  #[serde(flatten)]
  pub(crate) unparsed: Unparsed,
}

/// Instead of a simple ID string as object, Mastodon sends a nested tombstone for some reason,
/// so we need to handle that as well.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(untagged)]
pub(crate) enum IdOrNestedObject {
  Id(Url),
  NestedObject(NestedObject),
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub(crate) struct NestedObject {
  id: Url,
}

impl IdOrNestedObject {
  pub(crate) fn id(&self) -> &Url {
    match self {
      IdOrNestedObject::Id(i) => i,
      IdOrNestedObject::NestedObject(n) => &n.id,
    }
  }
}
