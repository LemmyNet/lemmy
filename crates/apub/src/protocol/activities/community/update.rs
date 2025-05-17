use crate::{
  objects::{community::ApubCommunity, person::ApubPerson},
  protocol::{objects::group::Group, InCommunity},
};
use activitypub_federation::{
  config::Data,
  fetch::object_id::ObjectId,
  kinds::activity::UpdateType,
  protocol::helpers::deserialize_one_or_many,
};
use lemmy_api_common::context::LemmyContext;
use lemmy_utils::error::LemmyResult;
use serde::{Deserialize, Serialize};
use url::Url;

/// This activity is received from a remote community mod, and updates the description or other
/// fields of a local community.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateCommunity {
  pub(crate) actor: ObjectId<ApubPerson>,
  #[serde(deserialize_with = "deserialize_one_or_many")]
  pub(crate) to: Vec<Url>,
  // TODO: would be nice to use a separate struct here, which only contains the fields updated here
  pub(crate) object: Box<Group>,
  #[serde(deserialize_with = "deserialize_one_or_many")]
  pub(crate) cc: Vec<Url>,
  #[serde(rename = "type")]
  pub(crate) kind: UpdateType,
  pub(crate) id: Url,
}

#[async_trait::async_trait]
impl InCommunity for UpdateCommunity {
  async fn community(&self, context: &Data<LemmyContext>) -> LemmyResult<ApubCommunity> {
    let community: ApubCommunity = self.object.id.clone().dereference(context).await?;
    Ok(community)
  }
}
