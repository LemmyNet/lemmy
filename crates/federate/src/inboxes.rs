use crate::util::LEMMY_TEST_FAST_FEDERATION;
use chrono::{DateTime, TimeZone, Utc};
use lemmy_api_common::context::LemmyContext;
use lemmy_db_schema::{
  newtypes::CommunityId,
  source::{activity::SentActivity, instance::Instance, site::Site},
};
use lemmy_db_views_actor::structs::CommunityFollowerView;
use lemmy_utils::error::LemmyResult;
use once_cell::sync::Lazy;
use reqwest::Url;
use std::collections::{HashMap, HashSet};

/// interval with which new additions to community_followers are queried.
///
/// The first time some user on an instance follows a specific remote community (or, more precisely:
/// the first time a (followed_community_id, follower_inbox_url) tuple appears), this delay limits
/// the maximum time until the follow actually results in activities from that community id being
/// sent to that inbox url. This delay currently needs to not be too small because the DB load is
/// currently fairly high because of the current structure of storing inboxes for every person, not
/// having a separate list of shared_inboxes, and the architecture of having every instance queue be
/// fully separate. (see https://github.com/LemmyNet/lemmy/issues/3958)
static FOLLOW_ADDITIONS_RECHECK_DELAY: Lazy<chrono::TimeDelta> = Lazy::new(|| {
  if *LEMMY_TEST_FAST_FEDERATION {
    chrono::TimeDelta::try_seconds(1).expect("TimeDelta out of bounds")
  } else {
    chrono::TimeDelta::try_minutes(2).expect("TimeDelta out of bounds")
  }
});

/// The same as FOLLOW_ADDITIONS_RECHECK_DELAY, but triggering when the last person on an instance
/// unfollows a specific remote community. This is expected to happen pretty rarely and updating it
/// in a timely manner is not too important.
static FOLLOW_REMOVALS_RECHECK_DELAY: Lazy<chrono::TimeDelta> =
  Lazy::new(|| chrono::TimeDelta::try_hours(1).expect("TimeDelta out of bounds"));

pub(crate) struct CommunityInboxCollector {
  instance: Instance,
  // load site lazily because if an instance is first seen due to being on allowlist,
  // the corresponding row in `site` may not exist yet since that is only added once
  // `fetch_instance_actor_for_object` is called.
  // (this should be unlikely to be relevant outside of the federation tests)
  // TODO: use lazy
  site_loaded: bool,
  site: Option<Site>,
  followed_communities: HashMap<CommunityId, HashSet<Url>>,
  last_communities_fetch_full: DateTime<Utc>,
  last_communities_fetch_incr: DateTime<Utc>,
}

impl CommunityInboxCollector {
  pub fn new(instance: Instance) -> Self {
    Self {
      instance,
      site_loaded: false,
      site: None,
      followed_communities: HashMap::new(),
      last_communities_fetch_full: Utc.timestamp_nanos(0),
      last_communities_fetch_incr: Utc.timestamp_nanos(0),
    }
  }

  /// get inbox urls of sending the given activity to the given instance
  /// most often this will return 0 values (if instance doesn't care about the activity)
  /// or 1 value (the shared inbox)
  /// > 1 values only happens for non-lemmy software
  pub async fn get_inbox_urls(
    &mut self,
    activity: &SentActivity,
    context: &LemmyContext,
  ) -> LemmyResult<HashSet<Url>> {
    let mut inbox_urls: HashSet<Url> = HashSet::new();

    if activity.send_all_instances {
      if !self.site_loaded {
        self.site = Site::read_from_instance_id(&mut context.pool(), self.instance.id).await?;
        self.site_loaded = true;
      }
      if let Some(site) = &self.site {
        // Nutomic: Most non-lemmy software wont have a site row. That means it cant handle these
        // activities. So handling it like this is fine.
        inbox_urls.insert(site.inbox_url.inner().clone());
      }
    }
    if let Some(t) = &activity.send_community_followers_of {
      if let Some(urls) = self.followed_communities.get(t) {
        inbox_urls.extend(urls.iter().cloned());
      }
    }
    inbox_urls.extend(
      activity
        .send_inboxes
        .iter()
        .filter_map(std::option::Option::as_ref)
        .filter(|&u| (u.domain() == Some(&self.instance.domain)))
        .map(|u| u.inner().clone()),
    );
    Ok(inbox_urls)
  }

  pub async fn update_communities(&mut self, context: &LemmyContext) -> LemmyResult<()> {
    // update to time before fetch to ensure overlap. subtract 10s to ensure overlap even if
    // published date is not exact
    let updated_fetch =
      Utc::now() - chrono::TimeDelta::try_seconds(10).expect("TimeDelta out of bounds");

    let full_fetch = Utc::now() - self.last_communities_fetch_full;
    if full_fetch > *FOLLOW_REMOVALS_RECHECK_DELAY {
      // process removals every hour
      self.followed_communities = self
        .get_communities(Utc.timestamp_nanos(0), context)
        .await?;
      self.last_communities_fetch_full = updated_fetch;
      self.last_communities_fetch_incr = self.last_communities_fetch_full;
    }
    let incr_fetch = Utc::now() - self.last_communities_fetch_incr;
    if incr_fetch > *FOLLOW_ADDITIONS_RECHECK_DELAY {
      // process additions every minute
      let added = self
        .get_communities(self.last_communities_fetch_incr, context)
        .await?;
      self.followed_communities.extend(added);
      self.last_communities_fetch_incr = updated_fetch;
    }
    Ok(())
  }

  /// get a list of local communities with the remote inboxes on the given instance that cares about
  /// them
  async fn get_communities(
    &mut self,
    last_fetch: DateTime<Utc>,
    context: &LemmyContext,
  ) -> LemmyResult<HashMap<CommunityId, HashSet<Url>>> {
    let followed = CommunityFollowerView::get_instance_followed_community_inboxes(
      &mut context.pool(),
      self.instance.id,
      last_fetch,
    )
    .await?;
    Ok(
      followed
        .into_iter()
        .fold(HashMap::new(), |mut map, (c, u)| {
          map.entry(c).or_default().insert(u.into());
          map
        }),
    )
  }
}
