use crate::protocol::{IdOrNestedObject, following::follow::Follow};
use activitypub_federation::{
  config::Data,
  fetch::object_id::ObjectId,
  kinds::activity::UndoType,
  protocol::helpers::deserialize_skip_error,
};
use lemmy_api_utils::context::LemmyContext;
use lemmy_apub_objects::{
  objects::{UserOrCommunity, community::ApubCommunity, person::ApubPerson},
  utils::protocol::InCommunity,
};
use lemmy_utils::error::LemmyResult;
use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UndoFollow {
  pub(crate) actor: ObjectId<UserOrCommunity>,
  /// Optional, for compatibility with platforms that always expect recipient field
  #[serde(deserialize_with = "deserialize_skip_error", default)]
  pub(crate) to: Option<[ObjectId<ApubPerson>; 1]>,
  pub(crate) object: IdOrNestedObject<Follow>,
  #[serde(rename = "type")]
  pub(crate) kind: UndoType,
  pub(crate) id: Url,
}

impl InCommunity for UndoFollow {
  async fn community(&self, context: &Data<LemmyContext>) -> LemmyResult<ApubCommunity> {
    let follow: Follow = self.object.dereference(context).await?;
    follow.community(context).await
  }
}
