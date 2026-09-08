use super::report::Report;
use crate::protocol::IdOrNestedObject;
use activitypub_federation::{
  config::Data,
  fetch::object_id::ObjectId,
  protocol::{helpers::deserialize_one, verification::verify_urls_match},
};
use either::Either;
use lemmy_api_utils::context::LemmyContext;
use lemmy_apub_objects::{
  objects::{community::ApubCommunity, instance::ApubSite, person::ApubPerson},
  utils::protocol::InCommunity,
};
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
  pub(crate) to: [ObjectId<Either<ApubSite, ApubCommunity>>; 1],
  pub(crate) object: IdOrNestedObject<Report>,
  #[serde(rename = "type")]
  pub(crate) kind: ResolveType,
  pub(crate) id: Url,
  pub(crate) audience: Option<ObjectId<ApubCommunity>>,
}

impl InCommunity for ResolveReport {
  async fn community(&self, context: &Data<LemmyContext>) -> LemmyResult<ApubCommunity> {
    let object = self.object.dereference(context).await?;
    let community = object.community(context).await?;
    if let Some(audience) = &self.audience {
      verify_urls_match(audience.inner(), community.ap_id.inner())?;
    }
    Ok(community)
  }
}
