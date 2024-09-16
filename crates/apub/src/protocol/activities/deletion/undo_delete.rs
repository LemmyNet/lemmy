use crate::{
  activities::verify_community_matches,
  objects::{community::ApubCommunity, person::ApubPerson},
  protocol::{activities::deletion::delete::Delete, InCommunity},
};
use activitypub_federation::{
  config::Data,
  fetch::object_id::ObjectId,
  kinds::activity::UndoType,
  protocol::helpers::deserialize_one_or_many,
};
use lemmy_api_common::context::LemmyContext;
use lemmy_utils::error::LemmyResult;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use url::Url;

#[skip_serializing_none]
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UndoDelete {
  pub(crate) actor: ObjectId<ApubPerson>,
  #[serde(deserialize_with = "deserialize_one_or_many")]
  pub(crate) to: Vec<Url>,
  pub(crate) object: Delete,
  #[serde(rename = "type")]
  pub(crate) kind: UndoType,
  pub(crate) id: Url,
  pub(crate) audience: Option<ObjectId<ApubCommunity>>,

  #[serde(deserialize_with = "deserialize_one_or_many", default)]
  #[serde(skip_serializing_if = "Vec::is_empty")]
  pub(crate) cc: Vec<Url>,
}

#[async_trait::async_trait]
impl InCommunity for UndoDelete {
  async fn community(&self, context: &Data<LemmyContext>) -> LemmyResult<ApubCommunity> {
    let community = self.object.community(context).await?;
    if let Some(audience) = &self.audience {
      verify_community_matches(audience, community.actor_id.clone())?;
    }
    Ok(community)
  }
}
