use crate::{
    activities::verify_community_matches,
    local_instance,
    mentions::NoteTags,
    objects::{community::ApubCommunity, person::ApubPerson},
    protocol::{activities::CreateOrUpdateType, objects::note::Note, InCommunity},
};
use activitypub_federation::{core::object_id::ObjectId, deser::helpers::deserialize_one_or_many};
use lemmy_api_common::context::LemmyContext;
use lemmy_db_schema::{source::community::Community, traits::Crud};
use lemmy_utils::error::LemmyError;
use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateOrUpdateNote {
  pub(crate) actor: ObjectId<ApubPerson>,
  #[serde(deserialize_with = "deserialize_one_or_many")]
  pub(crate) to: Vec<Url>,
  pub(crate) object: Note,
  #[serde(deserialize_with = "deserialize_one_or_many")]
  pub(crate) cc: Vec<Url>,
  #[serde(default)]
  pub(crate) tag: Vec<NoteTags>,
  #[serde(rename = "type")]
  pub(crate) kind: CreateOrUpdateType,
  pub(crate) id: Url,
  pub(crate) audience: Option<ObjectId<ApubCommunity>>,
}

#[async_trait::async_trait(?Send)]
impl InCommunity for CreateOrUpdateNote {
  async fn community(
    &self,
    context: &LemmyContext,
    request_counter: &mut i32,
  ) -> Result<ApubCommunity, LemmyError> {
    let post = self.object.get_parents(context, request_counter).await?.0;
    if let Some(audience) = &self.audience {
      let audience = audience
        .dereference(context, local_instance(context).await, request_counter)
        .await?;
      verify_community_matches(&audience, post.community_id)?;
      Ok(audience)
    } else {
      let community = Community::read(context.pool(), post.community_id).await?;
      Ok(community.into())
    }
  }
}
