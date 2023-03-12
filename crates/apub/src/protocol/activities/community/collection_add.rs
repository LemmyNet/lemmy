use crate::{
  activities::verify_community_matches,
  objects::{community::ApubCommunity, person::ApubPerson},
  protocol::InCommunity,
};
use activitypub_federation::{core::object_id::ObjectId, deser::helpers::deserialize_one_or_many};
use activitystreams_kinds::activity::AddType;
use lemmy_api_common::context::LemmyContext;
use lemmy_db_schema::source::community::Community;
use lemmy_utils::error::LemmyError;
use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CollectionAdd {
  pub(crate) actor: ObjectId<ApubPerson>,
  #[serde(deserialize_with = "deserialize_one_or_many")]
  pub(crate) to: Vec<Url>,
  pub(crate) object: Url,
  pub(crate) target: Url,
  #[serde(deserialize_with = "deserialize_one_or_many")]
  pub(crate) cc: Vec<Url>,
  #[serde(rename = "type")]
  pub(crate) kind: AddType,
  pub(crate) id: Url,
  pub(crate) audience: Option<ObjectId<ApubCommunity>>,
}

#[async_trait::async_trait(?Send)]
impl InCommunity for CollectionAdd {
  async fn community(
    &self,
    context: &LemmyContext,
    _request_counter: &mut i32,
  ) -> Result<ApubCommunity, LemmyError> {
    let (community, _) =
      Community::get_by_collection_url(context.pool(), &self.clone().target.into()).await?;
    if let Some(audience) = &self.audience {
      verify_community_matches(audience, community.actor_id.clone())?;
    }
    Ok(community.into())
  }
}
