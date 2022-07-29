use crate::{
  check_apub_id_valid_with_strictness,
  collections::{
    community_moderators::ApubCommunityModerators,
    community_outbox::ApubCommunityOutbox,
  },
  objects::{community::ApubCommunity, read_from_string_or_source_opt},
  protocol::{objects::Endpoints, ImageObject, Source},
};
use activitypub_federation::{
  core::{object_id::ObjectId, signatures::PublicKey},
  deser::helpers::deserialize_skip_error,
  utils::verify_domains_match,
};
use activitystreams_kinds::actor::GroupType;
use chrono::{DateTime, FixedOffset};
use lemmy_db_schema::{source::community::CommunityForm, utils::naive_now};
use lemmy_utils::{
  error::LemmyError,
  utils::{check_slurs, check_slurs_opt},
};
use lemmy_websocket::LemmyContext;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use url::Url;

#[skip_serializing_none]
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Group {
  #[serde(rename = "type")]
  pub(crate) kind: GroupType,
  pub(crate) id: ObjectId<ApubCommunity>,
  /// username, set at account creation and usually fixed after that
  pub(crate) preferred_username: String,
  pub(crate) inbox: Url,
  pub(crate) followers: Url,
  pub(crate) public_key: PublicKey,

  /// title
  pub(crate) name: Option<String>,
  pub(crate) summary: Option<String>,
  #[serde(deserialize_with = "deserialize_skip_error", default)]
  pub(crate) source: Option<Source>,
  pub(crate) icon: Option<ImageObject>,
  /// banner
  pub(crate) image: Option<ImageObject>,
  // lemmy extension
  pub(crate) sensitive: Option<bool>,
  // lemmy extension
  pub(crate) moderators: Option<ObjectId<ApubCommunityModerators>>,
  // lemmy extension
  pub(crate) posting_restricted_to_mods: Option<bool>,
  pub(crate) outbox: ObjectId<ApubCommunityOutbox>,
  pub(crate) endpoints: Option<Endpoints>,
  pub(crate) published: Option<DateTime<FixedOffset>>,
  pub(crate) updated: Option<DateTime<FixedOffset>>,
}

impl Group {
  pub(crate) async fn verify(
    &self,
    expected_domain: &Url,
    context: &LemmyContext,
  ) -> Result<(), LemmyError> {
    check_apub_id_valid_with_strictness(self.id.inner(), true, context.settings())?;
    verify_domains_match(expected_domain, self.id.inner())?;

    let slur_regex = &context.settings().slur_regex();
    check_slurs(&self.preferred_username, slur_regex)?;
    check_slurs_opt(&self.name, slur_regex)?;
    let description = read_from_string_or_source_opt(&self.summary, &None, &self.source);
    check_slurs_opt(&description, slur_regex)?;
    Ok(())
  }

  pub(crate) fn into_form(self) -> CommunityForm {
    CommunityForm {
      name: self.preferred_username.clone(),
      title: self.name.unwrap_or(self.preferred_username),
      description: Some(read_from_string_or_source_opt(
        &self.summary,
        &None,
        &self.source,
      )),
      removed: None,
      published: self.published.map(|u| u.naive_local()),
      updated: self.updated.map(|u| u.naive_local()),
      deleted: None,
      nsfw: Some(self.sensitive.unwrap_or(false)),
      actor_id: Some(self.id.into()),
      local: Some(false),
      private_key: None,
      hidden: Some(false),
      public_key: Some(self.public_key.public_key_pem),
      last_refreshed_at: Some(naive_now()),
      icon: Some(self.icon.map(|i| i.url.into())),
      banner: Some(self.image.map(|i| i.url.into())),
      followers_url: Some(self.followers.into()),
      inbox_url: Some(self.inbox.into()),
      shared_inbox_url: Some(self.endpoints.map(|e| e.shared_inbox.into())),
      posting_restricted_to_mods: self.posting_restricted_to_mods,
    }
  }
}
