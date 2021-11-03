use crate::{
  fetcher::object_id::ObjectId,
  objects::{community::ApubCommunity, person::ApubPerson},
};
use activitystreams::{activity::kind::BlockType, unparsed::Unparsed};
use lemmy_apub_lib::traits::ActivityFields;
use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Clone, Debug, Deserialize, Serialize, ActivityFields)]
#[serde(rename_all = "camelCase")]
pub struct BlockUserFromCommunity {
  pub(crate) actor: ObjectId<ApubPerson>,
  pub(crate) to: Vec<Url>,
  pub(crate) object: ObjectId<ApubPerson>,
  pub(crate) cc: Vec<Url>,
  pub(crate) target: ObjectId<ApubCommunity>,
  #[serde(rename = "type")]
  pub(crate) kind: BlockType,
  pub(crate) id: Url,
  #[serde(flatten)]
  pub(crate) unparsed: Unparsed,
}
