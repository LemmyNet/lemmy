use crate::{
  check_apub_id_valid_with_strictness,
  collections::{
    community_featured::ApubCommunityFeatured,
    community_moderators::ApubCommunityModerators,
    community_outbox::ApubCommunityOutbox,
  },
  fetch_local_site_data,
  objects::{community::ApubCommunity, read_from_string_or_source_opt},
  protocol::{
    objects::{Endpoints, LanguageTag},
    ImageObject,
    Source,
  },
};
use activitypub_federation::{
  fetch::{collection_id::CollectionId, object_id::ObjectId},
  kinds::actor::GroupType,
  protocol::{
    helpers::deserialize_skip_error,
    public_key::PublicKey,
    verification::verify_domains_match,
  },
};
use chrono::{DateTime, FixedOffset};
use lemmy_api_common::{context::LemmyContext, utils::local_site_opt_to_slur_regex};
use lemmy_db_schema::{
  newtypes::InstanceId,
  source::community::{CommunityInsertForm, CommunityUpdateForm},
  utils::naive_now,
};
use lemmy_utils::{
  error::LemmyError,
  utils::slurs::{check_slurs, check_slurs_opt},
};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use std::fmt::Debug;
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
  #[serde(deserialize_with = "deserialize_skip_error", default)]
  pub(crate) attributed_to: Option<CollectionId<ApubCommunityModerators>>,
  // lemmy extension
  pub(crate) posting_restricted_to_mods: Option<bool>,
  pub(crate) outbox: CollectionId<ApubCommunityOutbox>,
  pub(crate) endpoints: Option<Endpoints>,
  pub(crate) featured: Option<CollectionId<ApubCommunityFeatured>>,
  #[serde(default)]
  pub(crate) language: Vec<LanguageTag>,
  pub(crate) published: Option<DateTime<FixedOffset>>,
  pub(crate) updated: Option<DateTime<FixedOffset>>,
}

impl Group {
  pub(crate) async fn verify(
    &self,
    expected_domain: &Url,
    context: &LemmyContext,
  ) -> Result<(), LemmyError> {
    let local_site_data = fetch_local_site_data(context.pool()).await?;

    check_apub_id_valid_with_strictness(
      self.id.inner(),
      true,
      &local_site_data,
      context.settings(),
    )?;
    verify_domains_match(expected_domain, self.id.inner())?;

    let slur_regex = &local_site_opt_to_slur_regex(&local_site_data.local_site);

    check_slurs(&self.preferred_username, slur_regex)?;
    check_slurs_opt(&self.name, slur_regex)?;
    let description = read_from_string_or_source_opt(&self.summary, &None, &self.source);
    check_slurs_opt(&description, slur_regex)?;
    Ok(())
  }

  pub(crate) fn into_insert_form(self, instance_id: InstanceId) -> CommunityInsertForm {
    CommunityInsertForm {
      name: self.preferred_username.clone(),
      title: self.name.unwrap_or(self.preferred_username),
      description: read_from_string_or_source_opt(&self.summary, &None, &self.source),
      removed: None,
      published: self.published.map(|u| u.naive_local()),
      updated: self.updated.map(|u| u.naive_local()),
      deleted: Some(false),
      nsfw: Some(self.sensitive.unwrap_or(false)),
      actor_id: Some(self.id.into()),
      local: Some(false),
      private_key: None,
      hidden: None,
      public_key: self.public_key.public_key_pem,
      last_refreshed_at: Some(naive_now()),
      icon: self.icon.map(|i| i.url.into()),
      banner: self.image.map(|i| i.url.into()),
      followers_url: Some(self.followers.into()),
      inbox_url: Some(self.inbox.into()),
      shared_inbox_url: self.endpoints.map(|e| e.shared_inbox.into()),
      moderators_url: self.attributed_to.map(Into::into),
      posting_restricted_to_mods: self.posting_restricted_to_mods,
      instance_id,
      featured_url: self.featured.map(Into::into),
    }
  }

  pub(crate) fn into_update_form(self) -> CommunityUpdateForm {
    CommunityUpdateForm {
      title: Some(self.name.unwrap_or(self.preferred_username)),
      description: Some(read_from_string_or_source_opt(
        &self.summary,
        &None,
        &self.source,
      )),
      removed: None,
      published: self.published.map(|u| u.naive_local()),
      updated: Some(self.updated.map(|u| u.naive_local())),
      deleted: None,
      nsfw: Some(self.sensitive.unwrap_or(false)),
      actor_id: Some(self.id.into()),
      local: None,
      private_key: None,
      hidden: None,
      public_key: Some(self.public_key.public_key_pem),
      last_refreshed_at: Some(naive_now()),
      icon: Some(self.icon.map(|i| i.url.into())),
      banner: Some(self.image.map(|i| i.url.into())),
      followers_url: Some(self.followers.into()),
      inbox_url: Some(self.inbox.into()),
      shared_inbox_url: Some(self.endpoints.map(|e| e.shared_inbox.into())),
      moderators_url: self.attributed_to.map(Into::into),
      posting_restricted_to_mods: self.posting_restricted_to_mods,
      featured_url: self.featured.map(Into::into),
    }
  }
}
