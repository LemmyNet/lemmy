use crate::{
  objects::community::ApubCommunity,
  protocol::{activities::following::follow::FollowCommunity, Unparsed},
};
use activitystreams_kinds::activity::AcceptType;
use lemmy_apub_lib::object_id::ObjectId;
use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AcceptFollowCommunity {
  pub(crate) actor: ObjectId<ApubCommunity>,
  pub(crate) object: FollowCommunity,
  #[serde(rename = "type")]
  pub(crate) kind: AcceptType,
  pub(crate) id: Url,
  #[serde(flatten)]
  pub(crate) unparsed: Unparsed,
}
