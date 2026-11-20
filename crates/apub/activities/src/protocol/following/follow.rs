use activitypub_federation::{
  config::Data,
  fetch::object_id::ObjectId,
  kinds::activity::FollowType,
  protocol::helpers::deserialize_skip_error,
};
use either::Either;
use lemmy_api_utils::context::LemmyContext;
use lemmy_apub_objects::{
  objects::{UserOrCommunity, UserOrCommunityOrMulti, community::ApubCommunity},
  utils::protocol::{Id, InCommunity},
};
use lemmy_utils::error::{LemmyErrorType, LemmyResult};
use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Follow {
  pub(crate) actor: ObjectId<UserOrCommunity>,
  /// Optional, for compatibility with platforms that always expect recipient field
  #[serde(deserialize_with = "deserialize_skip_error", default)]
  pub(crate) to: Option<[ObjectId<UserOrCommunityOrMulti>; 1]>,
  pub(crate) object: ObjectId<UserOrCommunityOrMulti>,
  #[serde(rename = "type")]
  pub(crate) kind: FollowType,
  pub(crate) id: Url,
}

impl InCommunity for Follow {
  async fn community(&self, context: &Data<LemmyContext>) -> LemmyResult<ApubCommunity> {
    match self.object.dereference(context).await? {
      Either::Right(Either::Left(community)) => Ok(community),
      Either::Left(_) | Either::Right(Either::Right(_)) => Err(LemmyErrorType::NotFound.into()),
    }
  }
}

impl Id for Follow {
  fn id(&self) -> &Url {
    &self.id
  }
}
