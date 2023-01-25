use crate::{
  activities::verify_community_matches,
  fetcher::post_or_comment::PostOrComment,
  local_instance,
  objects::{community::ApubCommunity, person::ApubPerson},
  protocol::InCommunity,
};
use activitypub_federation::core::object_id::ObjectId;
use lemmy_api_common::context::LemmyContext;
use lemmy_utils::error::LemmyError;
use serde::{Deserialize, Serialize};
use std::convert::TryFrom;
use strum_macros::Display;
use url::Url;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Vote {
  pub(crate) actor: ObjectId<ApubPerson>,
  pub(crate) object: ObjectId<PostOrComment>,
  #[serde(rename = "type")]
  pub(crate) kind: VoteType,
  pub(crate) id: Url,
  pub(crate) audience: Option<ObjectId<ApubCommunity>>,
}

#[derive(Clone, Debug, Display, Deserialize, Serialize, PartialEq, Eq)]
pub enum VoteType {
  Like,
  Dislike,
}

impl TryFrom<i16> for VoteType {
  type Error = LemmyError;

  fn try_from(value: i16) -> Result<Self, Self::Error> {
    match value {
      1 => Ok(VoteType::Like),
      -1 => Ok(VoteType::Dislike),
      _ => Err(LemmyError::from_message("invalid vote value")),
    }
  }
}

impl From<&VoteType> for i16 {
  fn from(value: &VoteType) -> i16 {
    match value {
      VoteType::Like => 1,
      VoteType::Dislike => -1,
    }
  }
}

#[async_trait::async_trait(?Send)]
impl InCommunity for Vote {
  async fn community(
    &self,
    context: &LemmyContext,
    request_counter: &mut i32,
  ) -> Result<ApubCommunity, LemmyError> {
    let local_instance = local_instance(context).await;
    let object_community = self
      .object
      .dereference(context, local_instance, request_counter)
      .await?
      .community(context, request_counter)
      .await?;
    if let Some(audience) = &self.audience {
      let audience = audience
        .dereference(context, local_instance, request_counter)
        .await?;
      verify_community_matches(&audience, object_community.id)?;
      Ok(audience)
    } else {
      Ok(object_community)
    }
  }
}
