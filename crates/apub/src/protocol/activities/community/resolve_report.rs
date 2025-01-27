use super::report::Report;
use crate::{
  objects::{community::ApubCommunity, person::ApubPerson},
  protocol::InCommunity,
};
use activitypub_federation::{
  config::Data,
  fetch::object_id::ObjectId,
  protocol::helpers::deserialize_one,
};
use lemmy_api_common::context::LemmyContext;
use lemmy_utils::error::LemmyResult;
use serde::{Deserialize, Serialize};
use strum::Display;
use url::Url;

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq, Display)]
pub enum ResolveType {
  Resolve,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ResolveReport {
  pub(crate) actor: ObjectId<ApubPerson>,
  #[serde(deserialize_with = "deserialize_one")]
  pub(crate) to: [ObjectId<ApubCommunity>; 1],
  pub(crate) object: Report,
  #[serde(rename = "type")]
  pub(crate) kind: ResolveType,
  pub(crate) id: Url,
}

#[async_trait::async_trait]
impl InCommunity for ResolveReport {
  async fn community(&self, context: &Data<LemmyContext>) -> LemmyResult<ApubCommunity> {
    self.object.community(context).await
  }
}
