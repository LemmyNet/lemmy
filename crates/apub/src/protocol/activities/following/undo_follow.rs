use crate::{
  objects::person::ApubPerson,
  protocol::{activities::following::follow::FollowCommunity, Unparsed},
};
use activitypub_federation::core::object_id::ObjectId;
use activitystreams_kinds::activity::UndoType;
use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UndoFollowCommunity {
  pub(crate) actor: ObjectId<ApubPerson>,
  pub(crate) object: FollowCommunity,
  #[serde(rename = "type")]
  pub(crate) kind: UndoType,
  pub(crate) id: Url,

  #[serde(flatten)]
  pub(crate) unparsed: Unparsed,
}
