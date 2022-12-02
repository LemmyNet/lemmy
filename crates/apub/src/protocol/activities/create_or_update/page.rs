use crate::{
  activities::verify_community_matches,
  local_instance,
  objects::{community::ApubCommunity, person::ApubPerson},
  protocol::{activities::CreateOrUpdateType, objects::page::Page, InCommunity},
};
use activitypub_federation::{core::object_id::ObjectId, deser::helpers::deserialize_one_or_many};
use lemmy_api_common::context::LemmyContext;
use lemmy_utils::error::LemmyError;
use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateOrUpdatePage {
  pub(crate) actor: ObjectId<ApubPerson>,
  #[serde(deserialize_with = "deserialize_one_or_many")]
  pub(crate) to: Vec<Url>,
  pub(crate) object: Page,
  #[serde(deserialize_with = "deserialize_one_or_many")]
  pub(crate) cc: Vec<Url>,
  #[serde(rename = "type")]
  pub(crate) kind: CreateOrUpdateType,
  pub(crate) id: Url,
  pub(crate) audience: Option<ObjectId<ApubCommunity>>,
}

#[async_trait::async_trait(?Send)]
impl InCommunity for CreateOrUpdatePage {
  async fn community(
    &self,
    context: &LemmyContext,
    request_counter: &mut i32,
  ) -> Result<ApubCommunity, LemmyError> {
    let object_community = self.object.community(context, request_counter).await?;
    if let Some(audience) = &self.audience {
      let audience = audience
        .dereference(context, local_instance(context).await, request_counter)
        .await?;
      verify_community_matches(&audience, object_community.id)?;
      Ok(audience)
    } else {
      Ok(object_community)
    }
  }
}
