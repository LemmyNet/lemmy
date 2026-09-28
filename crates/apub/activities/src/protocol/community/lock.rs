use crate::{post_or_comment_community, protocol::IdOrNestedObject};
use activitypub_federation::{
  config::Data,
  fetch::object_id::ObjectId,
  kinds::activity::UndoType,
  protocol::{helpers::deserialize_one_or_many, verification::verify_urls_match},
};
use lemmy_api_utils::context::LemmyContext;
use lemmy_apub_objects::{
  objects::{PostOrComment, community::ApubCommunity, person::ApubPerson},
  utils::protocol::{Id, InCommunity},
};
use lemmy_utils::error::LemmyResult;
use serde::{Deserialize, Serialize};
use strum::Display;
use url::Url;

#[derive(Clone, Debug, Display, Deserialize, Serialize)]
pub enum LockType {
  Lock,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LockPageOrNote {
  pub(crate) actor: ObjectId<ApubPerson>,
  #[serde(deserialize_with = "deserialize_one_or_many")]
  pub(crate) to: Vec<Url>,
  pub(crate) object: ObjectId<PostOrComment>,
  #[serde(deserialize_with = "deserialize_one_or_many")]
  pub(crate) cc: Vec<Url>,
  #[serde(rename = "type")]
  pub(crate) kind: LockType,
  pub(crate) id: Url,
  /// Summary is the reason for the lock.
  pub(crate) summary: Option<String>,
  pub(crate) audience: Option<ObjectId<ApubCommunity>>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UndoLockPageOrNote {
  pub(crate) actor: ObjectId<ApubPerson>,
  #[serde(deserialize_with = "deserialize_one_or_many")]
  pub(crate) to: Vec<Url>,
  pub(crate) object: IdOrNestedObject<LockPageOrNote>,
  #[serde(deserialize_with = "deserialize_one_or_many")]
  pub(crate) cc: Vec<Url>,
  #[serde(rename = "type")]
  pub(crate) kind: UndoType,
  pub(crate) id: Url,
  /// Summary is the reason for the lock.
  pub(crate) summary: Option<String>,
  pub(crate) audience: Option<ObjectId<ApubCommunity>>,
}

impl InCommunity for LockPageOrNote {
  async fn community(&self, context: &Data<LemmyContext>) -> LemmyResult<ApubCommunity> {
    let post_or_comment = self.object.dereference(context).await?;
    let community = post_or_comment_community(&post_or_comment, context).await?;
    if let Some(audience) = &self.audience {
      verify_urls_match(audience.inner(), community.ap_id.inner())?;
    }
    Ok(community.into())
  }
}

impl InCommunity for UndoLockPageOrNote {
  async fn community(&self, context: &Data<LemmyContext>) -> LemmyResult<ApubCommunity> {
    let object = self.object.dereference(context).await?;
    let community = object.community(context).await?;
    if let Some(audience) = &self.audience {
      verify_urls_match(audience.inner(), community.ap_id.inner())?;
    }
    Ok(community)
  }
}

impl Id for LockPageOrNote {
  fn id(&self) -> &Url {
    &self.id
  }
}
