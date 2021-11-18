use crate::{objects::person::ApubPerson, protocol::objects::tombstone::Tombstone};
use activitystreams::{activity::kind::DeleteType, primitives::OneOrMany, unparsed::Unparsed};
use lemmy_apub_lib::object_id::ObjectId;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use url::Url;

#[skip_serializing_none]
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Delete {
  pub(crate) actor: ObjectId<ApubPerson>,
  pub(crate) to: Option<OneOrMany<Url>>,
  pub(crate) object: Tombstone,
  #[serde(rename = "type")]
  pub(crate) kind: DeleteType,
  /// If summary is present, this is a mod action (Remove in Lemmy terms). Otherwise, its a user
  /// deleting their own content.
  pub(crate) summary: Option<String>,
  pub(crate) id: Url,
  #[serde(flatten)]
  pub(crate) unparsed: Unparsed,
}
