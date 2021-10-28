use crate::{
  collections::{
    community_moderators::ApubCommunityModerators,
    community_outbox::ApubCommunityOutbox,
  },
  fetcher::object_id::ObjectId,
  objects::get_summary_from_string_or_source,
  protocol::{ImageObject, Source},
};
use activitystreams::{
  actor::{kind::GroupType, Endpoints},
  unparsed::Unparsed,
};
use chrono::{DateTime, FixedOffset};
use lemmy_apub_lib::{signatures::PublicKey, verify::verify_domains_match};
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
  pub(crate) id: Url,
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
  pub(crate) endpoints: Endpoints<Url>,
  pub(crate) public_key: PublicKey,
  pub(crate) published: Option<DateTime<FixedOffset>>,
  pub(crate) updated: Option<DateTime<FixedOffset>>,
  #[serde(flatten)]
  pub(crate) unparsed: Unparsed,
}

impl Group {
  pub(crate) async fn from_apub_to_form(
    group: &Group,
    expected_domain: &Url,
    settings: &Settings,
  ) -> Result<CommunityForm, LemmyError> {
    verify_domains_match(expected_domain, &group.id)?;
    let name = group.preferred_username.clone();
    let title = group.name.clone();
    let description = get_summary_from_string_or_source(&group.summary, &group.source);
    let shared_inbox = group.endpoints.shared_inbox.clone().map(|s| s.into());

    let slur_regex = &settings.slur_regex();
    check_slurs(&name, slur_regex)?;
    check_slurs(&title, slur_regex)?;
    check_slurs_opt(&description, slur_regex)?;

    Ok(CommunityForm {
      name,
      title,
      description,
      removed: None,
      published: group.published.map(|u| u.naive_local()),
      updated: group.updated.map(|u| u.naive_local()),
      deleted: None,
      nsfw: Some(group.sensitive.unwrap_or(false)),
      actor_id: Some(group.id.clone().into()),
      local: Some(false),
      private_key: None,
      public_key: Some(group.public_key.public_key_pem.clone()),
      last_refreshed_at: Some(naive_now()),
      icon: Some(group.icon.clone().map(|i| i.url.into())),
      banner: Some(group.image.clone().map(|i| i.url.into())),
      followers_url: Some(group.followers.clone().into()),
      inbox_url: Some(group.inbox.clone().into()),
      shared_inbox_url: Some(shared_inbox),
    })
  }
}
