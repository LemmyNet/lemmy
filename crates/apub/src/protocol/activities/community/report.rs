use crate::{
  fetcher::post_or_comment::PostOrComment,
  objects::{community::ApubCommunity, person::ApubPerson},
  protocol::Unparsed,
};
use activitystreams_kinds::activity::FlagType;
use lemmy_apub_lib::{deser::deserialize_one, object_id::ObjectId};
use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Report {
  pub(crate) actor: ObjectId<ApubPerson>,
  #[serde(deserialize_with = "deserialize_one")]
  pub(crate) to: ObjectId<ApubCommunity>,
  pub(crate) object: ObjectId<PostOrComment>,
  pub(crate) summary: String,
  #[serde(rename = "type")]
  pub(crate) kind: FlagType,
  pub(crate) id: Url,

  #[serde(flatten)]
  pub(crate) unparsed: Unparsed,
}
