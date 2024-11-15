use activitypub_federation::kinds::object::NoteType;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use url::Url;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct NoteWrapper {
  pub(crate) r#type: NoteType,
  pub(crate) to: Option<Vec<Url>>,
  pub(crate) cc: Option<Vec<Url>>,
  #[serde(flatten)]
  other: Map<String, Value>,
}
