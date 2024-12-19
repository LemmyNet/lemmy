use activitypub_federation::kinds::object::NoteType;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use url::Url;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateOrUpdateNoteWrapper {
  pub(crate) object: NoteWrapper,
  pub(crate) id: Url,
  #[serde(default)]
  pub(crate) to: Vec<Url>,
  #[serde(default)]
  pub(crate) cc: Vec<Url>,
  pub(crate) actor: Url,
  #[serde(flatten)]
  other: Map<String, Value>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct NoteWrapper {
  pub(crate) r#type: NoteType,
  #[serde(flatten)]
  other: Map<String, Value>,
}
