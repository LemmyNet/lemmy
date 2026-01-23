use crate::{
  objects::community::ApubCommunity,
  protocol::tags::ApubCommunityTag,
  utils::protocol::{AttributedTo, Endpoints, ImageObject, LanguageTag, Source},
};
use activitypub_federation::{
  fetch::object_id::ObjectId,
  kinds::actor::GroupType,
  protocol::{
    helpers::{deserialize_last, deserialize_skip_error},
    public_key::PublicKey,
    values::MediaTypeHtml,
  },
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use std::fmt::Debug;
use url::Url;

#[skip_serializing_none]
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Group {
  #[serde(rename = "type")]
  pub(crate) kind: GroupType,
  pub id: ObjectId<ApubCommunity>,
  /// username, set at account creation and usually fixed after that
  pub preferred_username: String,
  pub inbox: Url,
  pub followers: Option<Url>,
  pub public_key: PublicKey,

  /// title
  pub name: Option<String>,
  // short instance description
  pub(crate) content: Option<String>,
  #[serde(deserialize_with = "deserialize_skip_error", default)]
  pub source: Option<Source>,
  pub(crate) media_type: Option<MediaTypeHtml>,
  // sidebar
  pub summary: Option<String>,
  #[serde(deserialize_with = "deserialize_last", default)]
  pub icon: Option<ImageObject>,
  /// banner
  #[serde(deserialize_with = "deserialize_last", default)]
  pub image: Option<ImageObject>,
  // lemmy extension
  pub sensitive: Option<bool>,
  #[serde(deserialize_with = "deserialize_skip_error", default)]
  pub attributed_to: Option<AttributedTo>,
  // lemmy extension
  pub posting_restricted_to_mods: Option<bool>,
  pub outbox: Url,
  pub endpoints: Option<Endpoints>,
  pub featured: Option<Url>,
  #[serde(default)]
  pub(crate) language: Vec<LanguageTag>,
  /// True if this is a private community
  pub(crate) manually_approves_followers: Option<bool>,
  pub published: Option<DateTime<Utc>>,
  pub updated: Option<DateTime<Utc>>,
  /// https://docs.joinmastodon.org/spec/activitypub/#discoverable
  pub(crate) discoverable: Option<bool>,
  #[serde(deserialize_with = "deserialize_skip_error", default)]
  pub(crate) tag: Vec<ApubCommunityTag>,
}
