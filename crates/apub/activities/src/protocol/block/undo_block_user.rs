use super::block_user::BlockUser;
use crate::protocol::IdOrNestedObject;
use activitypub_federation::{
  config::Data,
  fetch::object_id::ObjectId,
  kinds::activity::UndoType,
  protocol::{helpers::deserialize_one_or_many, verification::verify_urls_match},
};
use lemmy_api_utils::context::LemmyContext;
use lemmy_apub_objects::{
  objects::{community::ApubCommunity, person::ApubPerson},
  utils::protocol::InCommunity,
};
use lemmy_utils::error::LemmyResult;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use url::Url;

#[skip_serializing_none]
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UndoBlockUser {
  pub(crate) actor: ObjectId<ApubPerson>,
  #[serde(deserialize_with = "deserialize_one_or_many")]
  pub(crate) to: Vec<Url>,
  pub(crate) object: IdOrNestedObject<BlockUser>,
  #[serde(deserialize_with = "deserialize_one_or_many")]
  pub(crate) cc: Vec<Url>,
  #[serde(rename = "type")]
  pub(crate) kind: UndoType,
  pub(crate) id: Url,
  pub(crate) audience: Option<ObjectId<ApubCommunity>>,

  /// Quick and dirty solution.
  /// TODO: send a separate Delete activity instead
  pub(crate) restore_data: Option<bool>,
}

impl InCommunity for UndoBlockUser {
  async fn community(&self, context: &Data<LemmyContext>) -> LemmyResult<ApubCommunity> {
    let object = self.object.dereference(context).await?;
    let community = object.community(context).await?;
    if let Some(audience) = &self.audience {
      verify_urls_match(audience.inner(), community.ap_id.inner())?;
    }
    Ok(community)
  }
}
