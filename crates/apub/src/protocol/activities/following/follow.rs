use crate::{
  fetcher::object_id::ObjectId,
  objects::{community::ApubCommunity, person::ApubPerson},
};
use activitystreams::{activity::kind::FollowType, unparsed::Unparsed};
use lemmy_apub_lib::traits::ActivityFields;
use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Clone, Debug, Deserialize, Serialize, ActivityFields)]
#[serde(rename_all = "camelCase")]
pub struct FollowCommunity {
  pub(crate) actor: ObjectId<ApubPerson>,
  pub(crate) to: [ObjectId<ApubCommunity>; 1],
  pub(crate) object: ObjectId<ApubCommunity>,
  #[serde(rename = "type")]
  pub(crate) kind: FollowType,
  pub(crate) id: Url,
  #[serde(flatten)]
  pub(crate) unparsed: Unparsed,
}
