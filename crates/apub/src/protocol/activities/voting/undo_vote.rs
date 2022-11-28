use crate::{
  activities::verify_community_matches,
  local_instance,
  objects::{community::ApubCommunity, person::ApubPerson},
  protocol::{activities::voting::vote::Vote, InCommunity},
};
use activitypub_federation::core::object_id::ObjectId;
use activitystreams_kinds::activity::UndoType;
use lemmy_api_common::context::LemmyContext;
use lemmy_utils::error::LemmyError;
use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UndoVote {
  pub(crate) actor: ObjectId<ApubPerson>,
  pub(crate) object: Vote,
  #[serde(rename = "type")]
  pub(crate) kind: UndoType,
  pub(crate) id: Url,
  pub(crate) audience: Option<ObjectId<ApubCommunity>>,
}

#[async_trait::async_trait(?Send)]
impl InCommunity for UndoVote {
  async fn community(
    &self,
    context: &LemmyContext,
    request_counter: &mut i32,
  ) -> Result<ApubCommunity, LemmyError> {
    let local_instance = local_instance(context).await;
    let object_community = self.object.community(context, request_counter).await?;
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
