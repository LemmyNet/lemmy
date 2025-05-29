use activitypub_federation::{
  config::Data,
  fetch::object_id::ObjectId,
  kinds::activity::UpdateType,
  protocol::helpers::deserialize_one_or_many,
};
use either::Either;
use lemmy_api_common::{context::LemmyContext, LemmyErrorType};
use lemmy_apub_objects::{
  objects::{community::ApubCommunity, person::ApubPerson},
  protocol::{group::Group, multi_community::Feed},
  utils::protocol::InCommunity,
};
use lemmy_utils::error::LemmyResult;
use serde::{Deserialize, Serialize};
use url::Url;

/// This activity is received from a remote community mod, and updates the description or other
/// fields of a local community.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Update {
  pub(crate) actor: ObjectId<ApubPerson>,
  #[serde(deserialize_with = "deserialize_one_or_many")]
  pub(crate) to: Vec<Url>,
  // TODO: would be nice to use a separate struct here, which only contains the fields updated here
  pub(crate) object: Either<Group, Feed>,
  #[serde(deserialize_with = "deserialize_one_or_many")]
  pub(crate) cc: Vec<Url>,
  #[serde(rename = "type")]
  pub(crate) kind: UpdateType,
  pub(crate) id: Url,
}

impl InCommunity for Update {
  async fn community(&self, context: &Data<LemmyContext>) -> LemmyResult<ApubCommunity> {
    match &self.object {
      Either::Left(c) => {
        let community: ApubCommunity = c.id.clone().dereference(context).await?;
        Ok(community)
      }
      Either::Right(_) => Err(LemmyErrorType::NotFound.into()),
    }
  }
}
