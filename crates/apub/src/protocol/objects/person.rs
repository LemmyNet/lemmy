use crate::{
  objects::person::ApubPerson,
  protocol::{objects::Endpoints, ImageObject, SourceCompat},
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
  /// username, set at account creation and usually fixed after that
  pub(crate) preferred_username: String,
  pub(crate) inbox: Url,
  /// mandatory field in activitypub, lemmy currently serves an empty outbox
  pub(crate) outbox: Url,
  pub(crate) public_key: PublicKey,

  /// displayname
  pub(crate) name: Option<String>,
  pub(crate) summary: Option<String>,
  #[serde(default)]
  pub(crate) source: SourceCompat,
  /// user avatar
  pub(crate) icon: Option<ImageObject>,
  /// user banner
  pub(crate) image: Option<ImageObject>,
  pub(crate) matrix_user_id: Option<String>,
  pub(crate) endpoints: Option<Endpoints>,
  pub(crate) published: Option<DateTime<FixedOffset>>,
  pub(crate) updated: Option<DateTime<FixedOffset>>,
}
