use crate::protocol::objects::note_wrapper::NoteWrapper;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use url::Url;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateOrUpdateNoteWrapper {
  object: NoteWrapper,
  pub(crate) id: Url,
  pub(crate) actor: Url,
  pub(crate) to: Option<Vec<Url>>,
  pub(crate) cc: Option<Vec<Url>>,
  #[serde(flatten)]
  other: Map<String, Value>,
}
