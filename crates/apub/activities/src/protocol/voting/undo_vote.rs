use super::vote::Vote;
use activitypub_federation::{fetch::object_id::ObjectId, kinds::activity::UndoType};
use lemmy_apub_objects::objects::{community::ApubCommunity, person::ApubPerson};
use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UndoVote {
  pub(crate) actor: ObjectId<ApubPerson>,
  pub(crate) object: Vote,
  #[serde(rename = "type")]
  pub(crate) kind: UndoType,
  pub(crate) id: Url,
  pub(crate) audience: Option<ObjectId<ApubCommunity>>,
}
