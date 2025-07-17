use crate::activities::block::SiteOrCommunity;
use activitypub_federation::{
  config::Data,
  fetch::object_id::ObjectId,
  kinds::activity::BlockType,
  protocol::helpers::deserialize_one_or_many,
};
use anyhow::anyhow;
use chrono::{DateTime, Utc};
use lemmy_api_utils::context::LemmyContext;
use lemmy_apub_objects::{
  objects::{community::ApubCommunity, person::ApubPerson},
  utils::protocol::InCommunity,
};
use lemmy_utils::error::LemmyResult;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use url::Url;

#[skip_serializing_none]
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
  pub(crate) end_time: Option<DateTime<Utc>>,
}

impl InCommunity for BlockUser {
  async fn community(&self, context: &Data<LemmyContext>) -> LemmyResult<ApubCommunity> {
    let target = self.target.dereference(context).await?;
    let community = match target {
      SiteOrCommunity::Right(c) => c,
      SiteOrCommunity::Left(_) => return Err(anyhow!("activity is not in community").into()),
    };
    Ok(community)
  }
}
