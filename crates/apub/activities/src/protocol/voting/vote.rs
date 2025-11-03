use crate::post_or_comment_community;
use activitypub_federation::{config::Data, fetch::object_id::ObjectId};
use lemmy_api_utils::context::LemmyContext;
use lemmy_apub_objects::{
  objects::{PostOrComment, community::ApubCommunity, person::ApubPerson},
  utils::protocol::InCommunity,
};
use lemmy_utils::error::LemmyResult;
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

impl From<bool> for VoteType {
  fn from(value: bool) -> Self {
    if value {
      VoteType::Like
    } else {
      VoteType::Dislike
    }
  }
}

impl From<&VoteType> for bool {
  fn from(value: &VoteType) -> Self {
    value == &VoteType::Like
  }
}

impl InCommunity for Vote {
  async fn community(&self, context: &Data<LemmyContext>) -> LemmyResult<ApubCommunity> {
    let post_or_comment = self.object.dereference(context).await?;
    let community = post_or_comment_community(&post_or_comment, context).await?;
    Ok(community.into())
  }
}
