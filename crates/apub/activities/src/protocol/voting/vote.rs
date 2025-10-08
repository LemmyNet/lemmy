use crate::post_or_comment_community;
use activitypub_federation::{config::Data, fetch::object_id::ObjectId};
use lemmy_api_utils::context::LemmyContext;
use lemmy_apub_objects::{
  objects::{community::ApubCommunity, person::ApubPerson, PostOrComment},
  utils::protocol::InCommunity,
};
use lemmy_utils::error::{LemmyError, LemmyResult, UntranslatedError};
use serde::{Deserialize, Serialize};
use strum::Display;
use url::Url;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Vote {
  pub(crate) actor: ObjectId<ApubPerson>,
  pub(crate) object: ObjectId<PostOrComment>,
  #[serde(rename = "type")]
  pub(crate) kind: VoteType,
  pub(crate) id: Url,
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
      _ => Err(UntranslatedError::InvalidVoteValue.into()),
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

impl InCommunity for Vote {
  async fn community(&self, context: &Data<LemmyContext>) -> LemmyResult<ApubCommunity> {
    let post_or_comment = self.object.dereference(context).await?;
    let community = post_or_comment_community(&post_or_comment, context).await?;
    Ok(community.into())
  }
}
