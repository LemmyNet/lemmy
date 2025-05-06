use crate::{
  objects::community::ApubCommunity,
  utils::{
    functions::check_apub_id_valid_with_strictness,
    protocol::{AttributedTo, Endpoints, ImageObject, LanguageTag, Source},
  },
};
use activitypub_federation::{
  config::Data,
  fetch::object_id::ObjectId,
  kinds::actor::GroupType,
  protocol::{
    helpers::deserialize_skip_error,
    public_key::PublicKey,
    values::MediaTypeHtml,
    verification::verify_domains_match,
  },
};
use chrono::{DateTime, Utc};
use lemmy_api_common::{context::LemmyContext, utils::slur_regex};
use lemmy_utils::{
  error::LemmyResult,
  utils::slurs::{check_slurs, check_slurs_opt},
};
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
  // sidebar
  pub(crate) content: Option<String>,
  #[serde(deserialize_with = "deserialize_skip_error", default)]
  pub source: Option<Source>,
  pub(crate) media_type: Option<MediaTypeHtml>,
  // short instance description
  pub summary: Option<String>,
  #[serde(deserialize_with = "deserialize_skip_error", default)]
  pub icon: Option<ImageObject>,
  /// banner
  #[serde(deserialize_with = "deserialize_skip_error", default)]
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
}

impl Group {
  pub(crate) async fn verify(
    &self,
    expected_domain: &Url,
    context: &Data<LemmyContext>,
  ) -> LemmyResult<()> {
    check_apub_id_valid_with_strictness(self.id.inner(), true, context).await?;
    verify_domains_match(expected_domain, self.id.inner())?;

    let slur_regex = slur_regex(context).await?;

    check_slurs(&self.preferred_username, &slur_regex)?;
    check_slurs_opt(&self.name, &slur_regex)?;
    check_slurs_opt(&self.summary, &slur_regex)?;
    Ok(())
  }
}
