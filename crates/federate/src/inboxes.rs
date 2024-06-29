use crate::util::LEMMY_TEST_FAST_FEDERATION;
use anyhow::Result;
use chrono::{DateTime, TimeZone, Utc};
use lemmy_db_schema::{
  newtypes::{CommunityId, InstanceId},
  source::{activity::SentActivity, site::Site},
  utils::{ActualDbPool, DbPool},
};
use lemmy_db_views_actor::structs::CommunityFollowerView;
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
  // load site lazily because if an instance is first seen due to being on allowlist,
  // the corresponding row in `site` may not exist yet since that is only added once
  // `fetch_instance_actor_for_object` is called.
  // (this should be unlikely to be relevant outside of the federation tests)
  site_loaded: bool,
  site: Option<Site>,
  followed_communities: HashMap<CommunityId, HashSet<Url>>,
  last_full_communities_fetch: DateTime<Utc>,
  last_incremental_communities_fetch: DateTime<Utc>,
  instance_id: InstanceId,
  domain: String,
  pool: ActualDbPool,
}
impl CommunityInboxCollector {
  pub fn new(
    pool: ActualDbPool,
    instance_id: InstanceId,
    domain: String,
  ) -> CommunityInboxCollector {
    CommunityInboxCollector {
      pool,
      site_loaded: false,
      site: None,
      followed_communities: HashMap::new(),
      last_full_communities_fetch: Utc.timestamp_nanos(0),
      last_incremental_communities_fetch: Utc.timestamp_nanos(0),
      instance_id,
      domain,
    }
  }
  /// get inbox urls of sending the given activity to the given instance
  /// most often this will return 0 values (if instance doesn't care about the activity)
  /// or 1 value (the shared inbox)
  /// > 1 values only happens for non-lemmy software
  pub async fn get_inbox_urls(&mut self, activity: &SentActivity) -> Result<Vec<Url>> {
    let mut inbox_urls: HashSet<Url> = HashSet::new();

    if activity.send_all_instances {
      if !self.site_loaded {
        self.site = Site::read_from_instance_id(&mut self.pool(), self.instance_id).await?;
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
        // a similar filter also happens within the activitypub-federation crate. but that filter
        // happens much later - by doing it here, we can ensure that in the happy case, this
        // function returns 0 urls which means the system doesn't have to create a tokio
        // task for sending at all (since that task has a fair amount of overhead)
        .filter(|&u| (u.domain() == Some(&self.domain)))
        .map(|u| u.inner().clone()),
    );
    tracing::trace!(
      "get_inbox_urls: {:?}, send_inboxes: {:?}",
      inbox_urls,
      activity.send_inboxes
    );
    Ok(inbox_urls.into_iter().collect())
  }

  pub async fn update_communities(&mut self) -> Result<()> {
    if (Utc::now() - self.last_full_communities_fetch) > *FOLLOW_REMOVALS_RECHECK_DELAY {
      tracing::debug!("{}: fetching full list of communities", self.domain);
      // process removals every hour
      (self.followed_communities, self.last_full_communities_fetch) = self
        .get_communities(self.instance_id, Utc.timestamp_nanos(0))
        .await?;
      self.last_incremental_communities_fetch = self.last_full_communities_fetch;
    }
    if (Utc::now() - self.last_incremental_communities_fetch) > *FOLLOW_ADDITIONS_RECHECK_DELAY {
      // process additions every minute
      let (news, time) = self
        .get_communities(self.instance_id, self.last_incremental_communities_fetch)
        .await?;
      if !news.is_empty() {
        tracing::debug!(
          "{}: fetched {} incremental new followed communities",
          self.domain,
          news.len()
        );
      }
      self.followed_communities.extend(news);
      self.last_incremental_communities_fetch = time;
    }
    Ok(())
  }

  /// get a list of local communities with the remote inboxes on the given instance that cares about
  /// them
  async fn get_communities(
    &mut self,
    instance_id: InstanceId,
    last_fetch: DateTime<Utc>,
  ) -> Result<(HashMap<CommunityId, HashSet<Url>>, DateTime<Utc>)> {
    // update to time before fetch to ensure overlap. subtract 10s to ensure overlap even if
    // published date is not exact
    let new_last_fetch =
      Utc::now() - chrono::TimeDelta::try_seconds(10).expect("TimeDelta out of bounds");
    Ok((
      CommunityFollowerView::get_instance_followed_community_inboxes(
        &mut self.pool(),
        instance_id,
        last_fetch,
      )
      .await?
      .into_iter()
      .fold(HashMap::new(), |mut map, (c, u)| {
        map.entry(c).or_default().insert(u.into());
        map
      }),
      new_last_fetch,
    ))
  }
  fn pool(&self) -> DbPool<'_> {
    DbPool::Pool(&self.pool)
  }
}
