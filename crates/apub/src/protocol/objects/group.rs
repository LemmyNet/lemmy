use crate::{
  check_apub_id_valid_with_strictness,
  collections::{
    community_featured::ApubCommunityFeatured,
    community_follower::ApubCommunityFollower,
    community_moderators::ApubCommunityModerators,
    community_outbox::ApubCommunityOutbox,
  },
  local_site_data_cached,
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
use chrono::{DateTime, Utc};
use lemmy_api_common::{context::LemmyContext, utils::local_site_opt_to_slur_regex};
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
  pub(crate) id: ObjectId<ApubCommunity>,
  /// username, set at account creation and usually fixed after that
  pub(crate) preferred_username: String,
  pub(crate) inbox: Url,
  pub(crate) followers: CollectionId<ApubCommunityFollower>,
  pub(crate) public_key: PublicKey,

  /// title
  pub(crate) name: Option<String>,
  pub(crate) summary: Option<String>,
  #[serde(deserialize_with = "deserialize_skip_error", default)]
  pub(crate) source: Option<Source>,
  #[serde(deserialize_with = "deserialize_skip_error", default)]
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
  pub(crate) published: Option<DateTime<Utc>>,
  pub(crate) updated: Option<DateTime<Utc>>,
}

impl Group {
  pub(crate) async fn verify(
    &self,
    expected_domain: &Url,
    context: &LemmyContext,
  ) -> LemmyResult<()> {
    check_apub_id_valid_with_strictness(self.id.inner(), true, context).await?;
    verify_domains_match(expected_domain, self.id.inner())?;

    let local_site_data = local_site_data_cached(&mut context.pool()).await?;
    let slur_regex = &local_site_opt_to_slur_regex(&local_site_data.local_site);

    check_slurs(&self.preferred_username, slur_regex)?;
    check_slurs_opt(&self.name, slur_regex)?;
    let description = read_from_string_or_source_opt(&self.summary, &None, &self.source);
    check_slurs_opt(&description, slur_regex)?;
    Ok(())
  }
}
