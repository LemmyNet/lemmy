use crate::{
  activities::verify_community_matches,
  fetcher::post_or_comment::PostOrComment,
  local_instance,
  objects::{community::ApubCommunity, person::ApubPerson},
  protocol::InCommunity,
};
use activitypub_federation::{core::object_id::ObjectId, deser::helpers::deserialize_one};
use activitystreams_kinds::activity::FlagType;
use lemmy_api_common::context::LemmyContext;
use lemmy_utils::error::LemmyError;
use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Report {
  pub(crate) actor: ObjectId<ApubPerson>,
  #[serde(deserialize_with = "deserialize_one")]
  pub(crate) to: [ObjectId<ApubCommunity>; 1],
  pub(crate) object: ObjectId<PostOrComment>,
  pub(crate) summary: String,
  #[serde(rename = "type")]
  pub(crate) kind: FlagType,
  pub(crate) id: Url,
  pub(crate) audience: Option<ObjectId<ApubCommunity>>,
}

#[async_trait::async_trait(?Send)]
impl InCommunity for Report {
  async fn community(
    &self,
    context: &LemmyContext,
    request_counter: &mut i32,
  ) -> Result<ApubCommunity, LemmyError> {
    let to_community = self.to[0]
      .dereference(context, local_instance(context).await, request_counter)
      .await?;
    if let Some(audience) = &self.audience {
      let audience = audience
        .dereference(context, local_instance(context).await, request_counter)
        .await?;
      verify_community_matches(&audience, to_community.id)?;
      Ok(audience)
    } else {
      Ok(to_community)
    }
  }
}
