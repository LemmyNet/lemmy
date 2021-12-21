use crate::{
  objects::person::ApubPerson,
  protocol::{objects::Endpoints, ImageObject, Source, Unparsed},
};
use chrono::{DateTime, FixedOffset};
use lemmy_apub_lib::{object_id::ObjectId, signatures::PublicKey};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use url::Url;

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq)]
pub enum UserTypes {
  Person,
  Service,
}

#[skip_serializing_none]
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Person {
  #[serde(rename = "type")]
  pub(crate) kind: UserTypes,
  pub(crate) id: ObjectId<ApubPerson>,
  /// username, set at account creation and can never be changed
  pub(crate) preferred_username: String,
  /// displayname (can be changed at any time)
  pub(crate) name: Option<String>,
  pub(crate) summary: Option<String>,
  pub(crate) source: Option<Source>,
  /// user avatar
  pub(crate) icon: Option<ImageObject>,
  /// user banner
  pub(crate) image: Option<ImageObject>,
  pub(crate) matrix_user_id: Option<String>,
  pub(crate) inbox: Url,
  /// mandatory field in activitypub, currently empty in lemmy
  pub(crate) outbox: Url,
  pub(crate) endpoints: Endpoints,
  pub(crate) public_key: PublicKey,
  pub(crate) published: Option<DateTime<FixedOffset>>,
  pub(crate) updated: Option<DateTime<FixedOffset>>,
  pub(crate) ban_expires: Option<DateTime<FixedOffset>>,
  #[serde(flatten)]
  pub(crate) unparsed: Unparsed,
}
