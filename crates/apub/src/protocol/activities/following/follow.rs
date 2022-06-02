use crate::{
  objects::{community::ApubCommunity, person::ApubPerson},
  protocol::Unparsed,
};
use activitypub_federation::core::object_id::ObjectId;
use activitystreams_kinds::activity::FollowType;
use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FollowCommunity {
  pub(crate) actor: ObjectId<ApubPerson>,
  pub(crate) object: ObjectId<ApubCommunity>,
  #[serde(rename = "type")]
  pub(crate) kind: FollowType,
  pub(crate) id: Url,

  #[serde(flatten)]
  pub(crate) unparsed: Unparsed,
}
