use crate::{
  fetcher::object_id::ObjectId,
  objects::{community::ApubCommunity, person::ApubPerson},
  protocol::activities::following::follow::FollowCommunity,
};
use activitystreams::{activity::kind::UndoType, unparsed::Unparsed};
use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UndoFollowCommunity {
  pub(crate) actor: ObjectId<ApubPerson>,
  pub(crate) to: [ObjectId<ApubCommunity>; 1],
  pub(crate) object: FollowCommunity,
  #[serde(rename = "type")]
  pub(crate) kind: UndoType,
  pub(crate) id: Url,
  #[serde(flatten)]
  pub(crate) unparsed: Unparsed,
}
