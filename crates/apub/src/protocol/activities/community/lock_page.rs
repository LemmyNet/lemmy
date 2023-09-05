use crate::{
    activities::verify_community_matches,
    objects::{community::ApubCommunity, person::ApubPerson, post::ApubPost},
    protocol::InCommunity,
};
use activitypub_federation::{
    config::Data, fetch::object_id::ObjectId, kinds::activity::UndoType,
    protocol::helpers::deserialize_one_or_many,
};
use lemmy_api_common::context::LemmyContext;
use lemmy_db_schema::{source::community::Community, traits::Crud};
use lemmy_utils::error::LemmyError;
use serde::{Deserialize, Serialize};
use strum_macros::Display;
use url::Url;

#[derive(Clone, Debug, Deserialize, Serialize, Display)]
pub enum LockType {
    Lock,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LockPage {
    pub(crate) actor: ObjectId<ApubPerson>,
    #[serde(deserialize_with = "deserialize_one_or_many")]
    pub(crate) to: Vec<Url>,
    pub(crate) object: ObjectId<ApubPost>,
    #[serde(deserialize_with = "deserialize_one_or_many")]
    pub(crate) cc: Vec<Url>,
    #[serde(rename = "type")]
    pub(crate) kind: LockType,
    pub(crate) id: Url,
    pub(crate) audience: Option<ObjectId<ApubCommunity>>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UndoLockPage {
    pub(crate) actor: ObjectId<ApubPerson>,
    #[serde(deserialize_with = "deserialize_one_or_many")]
    pub(crate) to: Vec<Url>,
    pub(crate) object: LockPage,
    #[serde(deserialize_with = "deserialize_one_or_many")]
    pub(crate) cc: Vec<Url>,
    #[serde(rename = "type")]
    pub(crate) kind: UndoType,
    pub(crate) id: Url,
    pub(crate) audience: Option<ObjectId<ApubCommunity>>,
}

#[async_trait::async_trait]
impl InCommunity for LockPage {
    async fn community(&self, context: &Data<LemmyContext>) -> Result<ApubCommunity, LemmyError> {
        let post = self.object.dereference(context).await?;
        let community = Community::read(&mut context.pool(), post.community_id).await?;
        if let Some(audience) = &self.audience {
            verify_community_matches(audience, community.actor_id.clone())?;
        }
        Ok(community.into())
    }
}

#[async_trait::async_trait]
impl InCommunity for UndoLockPage {
    async fn community(&self, context: &Data<LemmyContext>) -> Result<ApubCommunity, LemmyError> {
        let community = self.object.community(context).await?;
        if let Some(audience) = &self.audience {
            verify_community_matches(audience, community.actor_id.clone())?;
        }
        Ok(community)
    }
}
