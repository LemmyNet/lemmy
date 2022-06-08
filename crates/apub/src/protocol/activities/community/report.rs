use crate::{
  fetcher::post_or_comment::PostOrComment,
  objects::{community::ApubCommunity, person::ApubPerson},
  protocol::Unparsed,
};
use activitypub_federation::{core::object_id::ObjectId, deser::helpers::deserialize_one};
use activitystreams_kinds::activity::FlagType;
use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Report {
  pub(crate) actor: ObjectId<ApubPerson>,
  #[serde(deserialize_with = "deserialize_one")]
  pub(crate) to: [ObjectId<ApubCommunity>; 1],
  pub(crate) object: ObjectId<PostOrComment>,
  pub(crate) summary: String,
  #[serde(rename = "type")]
  pub(crate) kind: FlagType,
  pub(crate) id: Url,

  #[serde(flatten)]
  pub(crate) unparsed: Unparsed,
}
