use crate::{
  activities::{deletion::DeletableObjects, verify_community_matches},
  objects::{community::ApubCommunity, person::ApubPerson},
  protocol::{objects::tombstone::Tombstone, IdOrNestedObject, InCommunity},
};
use activitypub_federation::{
  config::Data, fetch::object_id::ObjectId, kinds::activity::DeleteType,
  protocol::helpers::deserialize_one_or_many,
};
use anyhow::anyhow;
use lemmy_api_common::context::LemmyContext;
use lemmy_db_schema::{
  source::{community::Community, post::Post},
  traits::Crud,
};
use lemmy_utils::error::LemmyError;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use url::Url;

#[skip_serializing_none]
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Delete {
  pub(crate) actor: ObjectId<ApubPerson>,
  #[serde(deserialize_with = "deserialize_one_or_many")]
  pub(crate) to: Vec<Url>,
  pub(crate) object: IdOrNestedObject<Tombstone>,
  #[serde(rename = "type")]
  pub(crate) kind: DeleteType,
  pub(crate) id: Url,
  pub(crate) audience: Option<ObjectId<ApubCommunity>>,

  #[serde(deserialize_with = "deserialize_one_or_many")]
  #[serde(default)]
  #[serde(skip_serializing_if = "Vec::is_empty")]
  pub(crate) cc: Vec<Url>,
  /// If summary is present, this is a mod action (Remove in Lemmy terms). Otherwise, its a user
  /// deleting their own content.
  pub(crate) summary: Option<String>,
}

#[async_trait::async_trait]
impl InCommunity for Delete {
  async fn community(&self, context: &Data<LemmyContext>) -> Result<ApubCommunity, LemmyError> {
    let community_id = match DeletableObjects::read_from_db(self.object.id(), context).await? {
      DeletableObjects::Community(c) => c.id,
      DeletableObjects::Comment(c) => {
        let post = Post::read(&mut context.pool(), c.post_id).await?;
        post.community_id
      }
      DeletableObjects::Post(p) => p.community_id,
      DeletableObjects::PrivateMessage(_) => {
        return Err(anyhow!("Private message is not part of community").into())
      }
    };
    let community = Community::read(&mut context.pool(), community_id).await?;
    if let Some(audience) = &self.audience {
      verify_community_matches(audience, community.actor_id.clone())?;
    }
    Ok(community.into())
  }
}
