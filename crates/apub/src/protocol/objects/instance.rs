use crate::{
  objects::instance::ApubSite,
  protocol::{objects::LanguageTag, ImageObject, Source},
};
use activitypub_federation::{
  fetch::object_id::ObjectId,
  kinds::actor::ApplicationType,
  protocol::{helpers::deserialize_skip_error, public_key::PublicKey, values::MediaTypeHtml},
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use url::Url;

#[skip_serializing_none]
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Instance {
  #[serde(rename = "type")]
  pub(crate) kind: ApplicationType,
  pub(crate) id: ObjectId<ApubSite>,
  // site name
  pub(crate) name: String,
  pub(crate) inbox: Url,
  /// mandatory field in activitypub, lemmy currently serves an empty outbox
  pub(crate) outbox: Url,
  pub(crate) public_key: PublicKey,

  // sidebar
  pub(crate) content: Option<String>,
  #[serde(deserialize_with = "deserialize_skip_error", default)]
  pub(crate) source: Option<Source>,
  // short instance description
  pub(crate) summary: Option<String>,
  pub(crate) media_type: Option<MediaTypeHtml>,
  /// instance icon
  pub(crate) icon: Option<ImageObject>,
  /// instance banner
  pub(crate) image: Option<ImageObject>,
  #[serde(default)]
  pub(crate) language: Vec<LanguageTag>,
  pub(crate) published: DateTime<Utc>,
  pub(crate) updated: Option<DateTime<Utc>>,
}
