use crate::{
  check_is_apub_id_valid,
  collections::{
    community_moderators::ApubCommunityModerators,
    community_outbox::ApubCommunityOutbox,
  },
  objects::{community::ApubCommunity, get_summary_from_string_or_source},
  protocol::{objects::Endpoints, ImageObject, Source},
};
use activitystreams::{actor::kind::GroupType, unparsed::Unparsed};
use chrono::{DateTime, FixedOffset};
use lemmy_apub_lib::{object_id::ObjectId, signatures::PublicKey, verify::verify_domains_match};
use lemmy_db_schema::{naive_now, source::community::CommunityForm};
use lemmy_utils::{
  settings::structs::Settings,
  utils::{check_slurs, check_slurs_opt},
  LemmyError,
};
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
  /// username, set at account creation and can never be changed
  pub(crate) preferred_username: String,
  /// title (can be changed at any time)
  pub(crate) name: String,
  pub(crate) summary: Option<String>,
  pub(crate) source: Option<Source>,
  pub(crate) icon: Option<ImageObject>,
  /// banner
  pub(crate) image: Option<ImageObject>,
  // lemmy extension
  pub(crate) sensitive: Option<bool>,
  // lemmy extension
  pub(crate) moderators: Option<ObjectId<ApubCommunityModerators>>,
  pub(crate) inbox: Url,
  pub(crate) outbox: ObjectId<ApubCommunityOutbox>,
  pub(crate) followers: Url,
  pub(crate) endpoints: Endpoints,
  pub(crate) public_key: PublicKey,
  pub(crate) published: Option<DateTime<FixedOffset>>,
  pub(crate) updated: Option<DateTime<FixedOffset>>,
  #[serde(flatten)]
  pub(crate) unparsed: Unparsed,
}

impl Group {
  pub(crate) async fn into_form(
    self,
    expected_domain: &Url,
    settings: &Settings,
  ) -> Result<CommunityForm, LemmyError> {
    check_is_apub_id_valid(self.id.inner(), true, settings)?;
    verify_domains_match(expected_domain, self.id.inner())?;
    let name = self.preferred_username;
    let title = self.name;
    let description = get_summary_from_string_or_source(&self.summary, &self.source);
    let shared_inbox = self.endpoints.shared_inbox.map(|s| s.into());

    let slur_regex = &settings.slur_regex();
    check_slurs(&name, slur_regex)?;
    check_slurs(&title, slur_regex)?;
    check_slurs_opt(&description, slur_regex)?;

    Ok(CommunityForm {
      name,
      title,
      description,
      removed: None,
      published: self.published.map(|u| u.naive_local()),
      updated: self.updated.map(|u| u.naive_local()),
      deleted: None,
      nsfw: Some(self.sensitive.unwrap_or(false)),
      actor_id: Some(self.id.into()),
      local: Some(false),
      private_key: None,
      public_key: Some(self.public_key.public_key_pem),
      last_refreshed_at: Some(naive_now()),
      icon: Some(self.icon.map(|i| i.url.into())),
      banner: Some(self.image.map(|i| i.url.into())),
      followers_url: Some(self.followers.into()),
      inbox_url: Some(self.inbox.into()),
      shared_inbox_url: Some(shared_inbox),
    })
  }
}
