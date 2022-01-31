use crate::{activities::block::SiteOrCommunity, objects::person::ApubPerson, protocol::Unparsed};
use activitystreams_kinds::activity::BlockType;
use chrono::{DateTime, FixedOffset};
use lemmy_apub_lib::object_id::ObjectId;
use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BlockUser {
  pub(crate) actor: ObjectId<ApubPerson>,
  #[serde(deserialize_with = "crate::deserialize_one_or_many")]
  pub(crate) to: Vec<Url>,
  pub(crate) object: ObjectId<ApubPerson>,
  #[serde(deserialize_with = "crate::deserialize_one_or_many")]
  pub(crate) cc: Vec<Url>,
  pub(crate) target: ObjectId<SiteOrCommunity>,
  #[serde(rename = "type")]
  pub(crate) kind: BlockType,
  /// Quick and dirty solution.
  /// TODO: send a separate Delete activity instead
  pub(crate) remove_data: Option<bool>,
  pub(crate) id: Url,
  #[serde(flatten)]
  pub(crate) unparsed: Unparsed,
  pub(crate) expires: Option<DateTime<FixedOffset>>,
}
