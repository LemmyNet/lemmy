use super::report::Report;
use activitypub_federation::{fetch::object_id::ObjectId, protocol::helpers::deserialize_one};
use either::Either;
use lemmy_apub_objects::objects::{
  community::ApubCommunity,
  instance::ApubSite,
  person::ApubPerson,
};
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
  pub(crate) object: Report,
  #[serde(rename = "type")]
  pub(crate) kind: ResolveType,
  pub(crate) id: Url,
}
