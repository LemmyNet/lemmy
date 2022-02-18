use crate::{
  check_is_apub_id_valid,
  collections::{
    community_moderators::ApubCommunityModerators,
    community_outbox::ApubCommunityOutbox,
  },
  objects::{community::ApubCommunity, get_summary_from_string_or_source},
  protocol::{objects::Endpoints, ImageObject, Source},
};
use activitystreams_kinds::actor::GroupType;
use chrono::{DateTime, FixedOffset};
use lemmy_apub_lib::{object_id::ObjectId, signatures::PublicKey, verify::verify_domains_match};
use lemmy_db_schema::{naive_now, source::community::CommunityForm};
use lemmy_utils::{
  utils::{check_slurs, check_slurs_opt},
  LemmyError,
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
  /// displayname
  pub(crate) name: String,
  pub(crate) inbox: Url,
  pub(crate) followers: Url,
  pub(crate) public_key: PublicKey,

  pub(crate) summary: Option<String>,
  pub(crate) source: Option<Source>,
  pub(crate) icon: Option<ImageObject>,
  /// banner
  pub(crate) image: Option<ImageObject>,
  // lemmy extension
  pub(crate) sensitive: Option<bool>,
  // lemmy extension
  pub(crate) moderators: Option<ObjectId<ApubCommunityModerators>>,
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
    check_is_apub_id_valid(self.id.inner(), true, &context.settings())?;
    verify_domains_match(expected_domain, self.id.inner())?;

    let slur_regex = &context.settings().slur_regex();
    check_slurs(&self.preferred_username, slur_regex)?;
    check_slurs(&self.name, slur_regex)?;
    let description = get_summary_from_string_or_source(&self.summary, &self.source);
    check_slurs_opt(&description, slur_regex)?;
    Ok(())
  }

  pub(crate) fn into_form(self) -> CommunityForm {
    CommunityForm {
      name: self.preferred_username,
      title: self.name,
      description: get_summary_from_string_or_source(&self.summary, &self.source),
      removed: None,
      published: self.published.map(|u| u.naive_local()),
      updated: self.updated.map(|u| u.naive_local()),
      deleted: None,
      nsfw: Some(self.sensitive.unwrap_or(false)),
      actor_id: Some(self.id.into()),
      local: Some(false),
      private_key: None,
      hidden: Some(false),
      public_key: self.public_key.public_key_pem,
      last_refreshed_at: Some(naive_now()),
      icon: Some(self.icon.map(|i| i.url.into())),
      banner: Some(self.image.map(|i| i.url.into())),
      followers_url: Some(self.followers.into()),
      inbox_url: Some(self.inbox.into()),
      shared_inbox_url: Some(self.endpoints.map(|e| e.shared_inbox.into())),
    }
  }
}
