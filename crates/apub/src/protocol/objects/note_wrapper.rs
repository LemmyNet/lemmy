use crate::{fetcher::post_or_comment::PostOrComment, objects::community::ApubCommunity};
use activitypub_federation::{fetch::object_id::ObjectId, kinds::object::NoteType};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use url::Url;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct NoteWrapper {
  pub(crate) r#type: NoteType,
  pub(crate) in_reply_to: ObjectId<PostOrComment>,
  pub(crate) audience: Option<ObjectId<ApubCommunity>>,
  #[serde(flatten)]
  other: Map<String, Value>,
}
