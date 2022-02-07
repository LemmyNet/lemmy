use crate::{
  objects::instance::ApubSite,
  protocol::{ImageObject, Source, Unparsed},
};
use activitystreams_kinds::actor::ServiceType;
use chrono::{DateTime, FixedOffset};
use lemmy_apub_lib::{object_id::ObjectId, signatures::PublicKey, values::MediaTypeHtml};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use url::Url;

#[skip_serializing_none]
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Instance {
  #[serde(rename = "type")]
  pub(crate) kind: ServiceType,
  pub(crate) id: ObjectId<ApubSite>,
  // site name
  pub(crate) name: String,
  // sidebar
  pub(crate) content: Option<String>,
  pub(crate) source: Option<Source>,
  // short instance description
  pub(crate) summary: Option<String>,
  pub(crate) media_type: Option<MediaTypeHtml>,
  /// instance icon
  pub(crate) icon: Option<ImageObject>,
  /// instance banner
  pub(crate) image: Option<ImageObject>,
  pub(crate) inbox: Url,
  /// mandatory field in activitypub, currently empty in lemmy
  pub(crate) outbox: Url,
  pub(crate) public_key: PublicKey,
  pub(crate) published: DateTime<FixedOffset>,
  pub(crate) updated: Option<DateTime<FixedOffset>>,
  #[serde(flatten)]
  pub(crate) unparsed: Unparsed,
}
