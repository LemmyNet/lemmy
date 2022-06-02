use crate::{activities::block::SiteOrCommunity, objects::person::ApubPerson, protocol::Unparsed};
use activitypub_federation::{core::object_id::ObjectId, deser::helpers::deserialize_one_or_many};
use activitystreams_kinds::activity::BlockType;
use chrono::{DateTime, FixedOffset};
use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BlockUser {
  pub(crate) actor: ObjectId<ApubPerson>,
  #[serde(deserialize_with = "deserialize_one_or_many")]
  pub(crate) to: Vec<Url>,
  pub(crate) object: ObjectId<ApubPerson>,
  #[serde(deserialize_with = "deserialize_one_or_many")]
  pub(crate) cc: Vec<Url>,
  pub(crate) target: ObjectId<SiteOrCommunity>,
  #[serde(rename = "type")]
  pub(crate) kind: BlockType,
  pub(crate) id: Url,

  /// Quick and dirty solution.
  /// TODO: send a separate Delete activity instead
  pub(crate) remove_data: Option<bool>,
  /// block reason, written to mod log
  pub(crate) summary: Option<String>,
  pub(crate) expires: Option<DateTime<FixedOffset>>,
  #[serde(flatten)]
  pub(crate) unparsed: Unparsed,
}
