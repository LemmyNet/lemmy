use crate::{
  fetcher::{object_id::ObjectId, post_or_comment::PostOrComment},
  objects::{community::ApubCommunity, person::ApubPerson},
};
use activitystreams::{activity::kind::FlagType, unparsed::Unparsed};
use lemmy_apub_lib::traits::ActivityFields;
use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Clone, Debug, Deserialize, Serialize, ActivityFields)]
#[serde(rename_all = "camelCase")]
pub struct Report {
  pub(crate) actor: ObjectId<ApubPerson>,
  pub(crate) to: [ObjectId<ApubCommunity>; 1],
  pub(crate) object: ObjectId<PostOrComment>,
  pub(crate) summary: String,
  #[serde(rename = "type")]
  pub(crate) kind: FlagType,
  pub(crate) id: Url,
  #[serde(flatten)]
  pub(crate) unparsed: Unparsed,
}
