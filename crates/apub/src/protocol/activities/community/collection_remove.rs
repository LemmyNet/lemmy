use crate::{
  activities::verify_community_matches,
  objects::{community::ApubCommunity, person::ApubPerson},
  protocol::InCommunity,
};
use activitypub_federation::{
  config::Data,
  fetch::object_id::ObjectId,
  kinds::activity::RemoveType,
  protocol::helpers::deserialize_one_or_many,
};
use lemmy_api_common::context::LemmyContext;
use lemmy_db_schema::source::community::Community;
use lemmy_utils::{error::LemmyResult, LemmyErrorType};
use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CollectionRemove {
  pub(crate) actor: ObjectId<ApubPerson>,
  #[serde(deserialize_with = "deserialize_one_or_many")]
  pub(crate) to: Vec<Url>,
  pub(crate) object: Url,
  #[serde(deserialize_with = "deserialize_one_or_many")]
  pub(crate) cc: Vec<Url>,
  #[serde(rename = "type")]
  pub(crate) kind: RemoveType,
  pub(crate) target: Url,
  pub(crate) id: Url,
  pub(crate) audience: Option<ObjectId<ApubCommunity>>,
}

#[async_trait::async_trait]
impl InCommunity for CollectionRemove {
  async fn community(&self, context: &Data<LemmyContext>) -> LemmyResult<ApubCommunity> {
    let (community, _) =
      Community::get_by_collection_url(&mut context.pool(), &self.clone().target.into())
        .await?
        .ok_or(LemmyErrorType::CouldntFindCommunity)?;
    if let Some(audience) = &self.audience {
      verify_community_matches(audience, community.actor_id.clone())?;
    }
    Ok(community.into())
  }
}
