use crate::{
  fetcher::object_id::ObjectId,
  objects::{community::ApubCommunity, person::ApubPerson},
  protocol::activities::following::follow::FollowCommunity,
};
use activitystreams::{activity::kind::AcceptType, unparsed::Unparsed};
use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AcceptFollowCommunity {
  pub(crate) actor: ObjectId<ApubCommunity>,
  pub(crate) to: [ObjectId<ApubPerson>; 1],
  pub(crate) object: FollowCommunity,
  #[serde(rename = "type")]
  pub(crate) kind: AcceptType,
  pub(crate) id: Url,
  #[serde(flatten)]
  pub(crate) unparsed: Unparsed,
}
