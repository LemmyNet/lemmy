use crate::{
  activities::{block::SiteOrCommunity, verify_community_matches},
  local_instance,
  objects::{community::ApubCommunity, person::ApubPerson},
  protocol::InCommunity,
};
use activitypub_federation::{core::object_id::ObjectId, deser::helpers::deserialize_one_or_many};
use activitystreams_kinds::activity::BlockType;
use anyhow::anyhow;
use chrono::{DateTime, FixedOffset};
use lemmy_api_common::context::LemmyContext;
use lemmy_utils::error::LemmyError;
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
  pub(crate) audience: Option<ObjectId<ApubCommunity>>,

  /// Quick and dirty solution.
  /// TODO: send a separate Delete activity instead
  pub(crate) remove_data: Option<bool>,
  /// block reason, written to mod log
  pub(crate) summary: Option<String>,
  pub(crate) expires: Option<DateTime<FixedOffset>>,
}

#[async_trait::async_trait(?Send)]
impl InCommunity for BlockUser {
  async fn community(
    &self,
    context: &LemmyContext,
    request_counter: &mut i32,
  ) -> Result<ApubCommunity, LemmyError> {
    let target = self
      .target
      .dereference(context, local_instance(context).await, request_counter)
      .await?;
    let target_community = match target {
      SiteOrCommunity::Community(c) => c,
      SiteOrCommunity::Site(_) => return Err(anyhow!("activity is not in community").into()),
    };
    if let Some(audience) = &self.audience {
      let audience = audience
        .dereference(context, local_instance(context).await, request_counter)
        .await?;
      verify_community_matches(&audience, target_community.id)?;
      Ok(audience)
    } else {
      Ok(target_community)
    }
  }
}
