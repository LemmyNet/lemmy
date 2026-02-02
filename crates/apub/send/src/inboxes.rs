use crate::util::LEMMY_TEST_FAST_FEDERATION;
use chrono::{DateTime, TimeZone, Utc};
use lemmy_db_schema::{
  newtypes::CommunityId,
  source::{activity::SentActivity, site::Site},
};
use lemmy_db_schema_file::InstanceId;
use lemmy_db_views_community_follower::CommunityFollowerView;
use lemmy_diesel_utils::{
  connection::{ActualDbPool, DbPool},
  dburl::DbUrl,
};
use lemmy_utils::error::LemmyResult;
use reqwest::Url;
use std::{
  collections::{HashMap, HashSet},
  sync::LazyLock,
};

/// interval with which new additions to community_followers are queried.
///
/// The first time some user on an instance follows a specific remote community (or, more precisely:
/// the first time a (followed_community_id, follower_inbox_url) tuple appears), this delay limits
/// the maximum time until the follow actually results in activities from that community id being
/// sent to that inbox url. This delay currently needs to not be too small because the DB load is
/// currently fairly high because of the current structure of storing inboxes for every person, not
/// having a separate list of shared_inboxes, and the architecture of having every instance queue be
/// fully separate. (see https://github.com/LemmyNet/lemmy/issues/3958)
#[allow(clippy::expect_used)]
static FOLLOW_ADDITIONS_RECHECK_DELAY: LazyLock<chrono::TimeDelta> = LazyLock::new(|| {
  if *LEMMY_TEST_FAST_FEDERATION {
    chrono::TimeDelta::try_seconds(1).expect("TimeDelta out of bounds")
  } else {
    chrono::TimeDelta::try_minutes(2).expect("TimeDelta out of bounds")
  }
});
/// The same as FOLLOW_ADDITIONS_RECHECK_DELAY, but triggering when the last person on an instance
/// unfollows a specific remote community. This is expected to happen pretty rarely and updating it
/// in a timely manner is not too important.
#[allow(clippy::expect_used)]
static FOLLOW_REMOVALS_RECHECK_DELAY: LazyLock<chrono::TimeDelta> =
  LazyLock::new(|| chrono::TimeDelta::try_hours(1).expect("TimeDelta out of bounds"));

pub trait DataSource: Send + Sync {
  async fn read_site_from_instance_id(&self, instance_id: InstanceId) -> LemmyResult<Site>;
  async fn get_instance_followed_community_inboxes(
    &self,
    instance_id: InstanceId,
    last_fetch: DateTime<Utc>,
  ) -> LemmyResult<Vec<(CommunityId, DbUrl)>>;
}
pub struct DbDataSource {
  pool: ActualDbPool,
}

impl DbDataSource {
  pub fn new(pool: ActualDbPool) -> Self {
    Self { pool }
  }
}

impl DataSource for DbDataSource {
  async fn read_site_from_instance_id(&self, instance_id: InstanceId) -> LemmyResult<Site> {
    Site::read_from_instance_id(&mut DbPool::Pool(&self.pool), instance_id).await
  }

  async fn get_instance_followed_community_inboxes(
    &self,
    instance_id: InstanceId,
    last_fetch: DateTime<Utc>,
  ) -> LemmyResult<Vec<(CommunityId, DbUrl)>> {
    CommunityFollowerView::get_instance_followed_community_inboxes(
      &mut DbPool::Pool(&self.pool),
      instance_id,
      last_fetch,
    )
    .await
  }
}

pub(crate) struct CommunityInboxCollector<T: DataSource> {
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
  pub(crate) data_source: T,
}

pub type RealCommunityInboxCollector = CommunityInboxCollector<DbDataSource>;

impl<T: DataSource> CommunityInboxCollector<T> {
  pub fn new_real(
    pool: ActualDbPool,
    instance_id: InstanceId,
    domain: String,
  ) -> RealCommunityInboxCollector {
    CommunityInboxCollector::new(DbDataSource::new(pool), instance_id, domain)
  }
  pub fn new(
    data_source: T,
    instance_id: InstanceId,
    domain: String,
  ) -> CommunityInboxCollector<T> {
    CommunityInboxCollector {
      data_source,
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
  pub async fn get_inbox_urls(&mut self, activity: &SentActivity) -> LemmyResult<Vec<Url>> {
    let mut inbox_urls: HashSet<Url> = HashSet::new();

    if activity.send_all_instances {
      if !self.site_loaded {
        self.site = self
          .data_source
          .read_site_from_instance_id(self.instance_id)
          .await
          .ok();
        self.site_loaded = true;
      }
      if let Some(site) = &self.site {
        // Nutomic: Most non-lemmy software wont have a site row. That means it cant handle these
        // activities. So handling it like this is fine.
        inbox_urls.insert(site.inbox_url.inner().clone());
      }
    }
    if let Some(t) = &activity.send_community_followers_of
      && let Some(urls) = self.followed_communities.get(t)
    {
      inbox_urls.extend(urls.iter().cloned());
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
        .filter(|&u| u.domain() == Some(&self.domain))
        .map(|u| u.inner().clone()),
    );
    tracing::trace!(
      "get_inbox_urls: {:?}, send_inboxes: {:?}",
      inbox_urls,
      activity.send_inboxes
    );
    Ok(inbox_urls.into_iter().collect())
  }

  pub async fn update_communities(&mut self) -> LemmyResult<()> {
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
  ) -> LemmyResult<(HashMap<CommunityId, HashSet<Url>>, DateTime<Utc>)> {
    // update to time before fetch to ensure overlap. subtract some time to ensure overlap even if
    // published date is not exact
    let new_last_fetch = Utc::now() - *FOLLOW_ADDITIONS_RECHECK_DELAY / 2;

    let inboxes = self
      .data_source
      .get_instance_followed_community_inboxes(instance_id, last_fetch)
      .await?;

    let map: HashMap<CommunityId, HashSet<Url>> =
      inboxes.into_iter().fold(HashMap::new(), |mut map, (c, u)| {
        map.entry(c).or_default().insert(u.into());
        map
      });

    Ok((map, new_last_fetch))
  }
}

#[cfg(test)]
#[expect(clippy::indexing_slicing)]
mod tests {
  use super::*;
  use lemmy_db_schema::{
    newtypes::{ActivityId, CommunityId, SiteId},
    source::activity::SentActivity,
  };
  use lemmy_db_schema_file::{InstanceId, enums::ActorType};
  use lemmy_utils::error::LemmyResult;
  use mockall::mock;
  use serde_json::json;

  mock! {
      DataSource {}
      impl DataSource for DataSource {
          async fn read_site_from_instance_id(&self, instance_id: InstanceId) -> LemmyResult<Site>;
          async fn get_instance_followed_community_inboxes(
              &self,
              instance_id: InstanceId,
              last_fetch: DateTime<Utc>,
          ) -> LemmyResult<Vec<(CommunityId, DbUrl)>>;
      }
  }

  fn setup_collector() -> CommunityInboxCollector<MockDataSource> {
    let mock_data_source = MockDataSource::new();
    let instance_id = InstanceId(1);
    let domain = "example.com".to_string();
    CommunityInboxCollector::new(mock_data_source, instance_id, domain)
  }

  #[tokio::test]
  async fn test_get_inbox_urls_empty() -> LemmyResult<()> {
    let mut collector = setup_collector();
    let activity = SentActivity {
      id: ActivityId(1),
      ap_id: Url::parse("https://example.com/activities/1")?.into(),
      data: json!({}),
      sensitive: false,
      published_at: Utc::now(),
      send_inboxes: vec![],
      send_community_followers_of: None,
      send_all_instances: false,
      actor_type: ActorType::Person,
      actor_apub_id: None,
    };

    let result = collector.get_inbox_urls(&activity).await?;
    assert!(result.is_empty());

    Ok(())
  }

  #[tokio::test]
  async fn test_get_inbox_urls_send_all_instances() -> LemmyResult<()> {
    let mut collector = setup_collector();
    let site_inbox = Url::parse("https://example.com/inbox")?;
    let site = Site {
      id: SiteId(1),
      name: "Test Site".to_string(),
      sidebar: None,
      published_at: Utc::now(),
      updated_at: None,
      icon: None,
      banner: None,
      summary: None,
      ap_id: Url::parse("https://example.com/site")?.into(),
      last_refreshed_at: Utc::now(),
      inbox_url: site_inbox.clone().into(),
      private_key: None,
      public_key: "test_key".to_string(),
      instance_id: InstanceId(1),
      content_warning: None,
    };

    collector
      .data_source
      .expect_read_site_from_instance_id()
      .return_once(move |_| Ok(site));

    let activity = SentActivity {
      id: ActivityId(1),
      ap_id: Url::parse("https://example.com/activities/1")?.into(),
      data: json!({}),
      sensitive: false,
      published_at: Utc::now(),
      send_inboxes: vec![],
      send_community_followers_of: None,
      send_all_instances: true,
      actor_type: ActorType::Person,
      actor_apub_id: None,
    };

    let result = collector.get_inbox_urls(&activity).await?;
    assert_eq!(result.len(), 1);
    assert_eq!(result[0], site_inbox);

    Ok(())
  }

  #[tokio::test]
  async fn test_get_inbox_urls_community_followers() -> LemmyResult<()> {
    let mut collector = setup_collector();
    let community_id = CommunityId(1);
    let url1 = "https://follower1.example.com/inbox";
    let url2 = "https://follower2.example.com/inbox";

    collector
      .data_source
      .expect_get_instance_followed_community_inboxes()
      .return_once(move |_, _| {
        Ok(vec![
          (community_id, Url::parse(url1)?.into()),
          (community_id, Url::parse(url2)?.into()),
        ])
      });

    collector.update_communities().await?;

    let activity = SentActivity {
      id: ActivityId(1),
      ap_id: Url::parse("https://example.com/activities/1")?.into(),
      data: json!({}),
      sensitive: false,
      published_at: Utc::now(),
      send_inboxes: vec![],
      send_community_followers_of: Some(community_id),
      send_all_instances: false,
      actor_type: ActorType::Person,
      actor_apub_id: None,
    };

    let result = collector.get_inbox_urls(&activity).await?;
    assert_eq!(result.len(), 2);
    assert!(result.contains(&Url::parse(url1)?));
    assert!(result.contains(&Url::parse(url2)?));

    Ok(())
  }

  #[tokio::test]
  async fn test_get_inbox_urls_send_inboxes() -> LemmyResult<()> {
    let mut collector = setup_collector();
    collector.domain = "example.com".to_string();
    let inbox_user_1 = Url::parse("https://example.com/user1/inbox")?;
    let inbox_user_2 = Url::parse("https://example.com/user2/inbox")?;
    let other_domain_inbox = Url::parse("https://other-domain.com/user3/inbox")?;
    let activity = SentActivity {
      id: ActivityId(1),
      ap_id: Url::parse("https://example.com/activities/1")?.into(),
      data: json!({}),
      sensitive: false,
      published_at: Utc::now(),
      send_inboxes: vec![
        Some(inbox_user_1.clone().into()),
        Some(inbox_user_2.clone().into()),
        Some(other_domain_inbox.clone().into()),
      ],
      send_community_followers_of: None,
      send_all_instances: false,
      actor_type: ActorType::Person,
      actor_apub_id: None,
    };

    let result = collector.get_inbox_urls(&activity).await?;
    assert_eq!(result.len(), 2);
    assert!(result.contains(&inbox_user_1));
    assert!(result.contains(&inbox_user_2));
    assert!(!result.contains(&other_domain_inbox));

    Ok(())
  }

  #[tokio::test]
  async fn test_get_inbox_urls_combined() -> LemmyResult<()> {
    let mut collector = setup_collector();
    collector.domain = "example.com".to_string();
    let community_id = CommunityId(1);

    let site_inbox = Url::parse("https://example.com/site_inbox")?;
    let site = Site {
      id: SiteId(1),
      name: "Test Site".to_string(),
      sidebar: None,
      published_at: Utc::now(),
      updated_at: None,
      icon: None,
      banner: None,
      summary: None,
      ap_id: Url::parse("https://example.com/site")?.into(),
      last_refreshed_at: Utc::now(),
      inbox_url: site_inbox.clone().into(),
      private_key: None,
      public_key: "test_key".to_string(),
      instance_id: InstanceId(1),
      content_warning: None,
    };

    collector
      .data_source
      .expect_read_site_from_instance_id()
      .return_once(move |_| Ok(site));

    let subdomain_inbox = "https://follower.example.com/inbox";
    collector
      .data_source
      .expect_get_instance_followed_community_inboxes()
      .return_once(move |_, _| Ok(vec![(community_id, Url::parse(subdomain_inbox)?.into())]));

    collector.update_communities().await?;
    let user1_inbox = Url::parse("https://example.com/user1/inbox")?;
    let user2_inbox = Url::parse("https://other-domain.com/user2/inbox")?;
    let activity = SentActivity {
      id: ActivityId(1),
      ap_id: Url::parse("https://example.com/activities/1")?.into(),
      data: json!({}),
      sensitive: false,
      published_at: Utc::now(),
      send_inboxes: vec![
        Some(user1_inbox.clone().into()),
        Some(user2_inbox.clone().into()),
      ],
      send_community_followers_of: Some(community_id),
      send_all_instances: true,
      actor_type: ActorType::Person,
      actor_apub_id: None,
    };

    let result = collector.get_inbox_urls(&activity).await?;
    assert_eq!(result.len(), 3);
    assert!(result.contains(&site_inbox));
    assert!(result.contains(&Url::parse(subdomain_inbox)?));
    assert!(result.contains(&user1_inbox));
    assert!(!result.contains(&user2_inbox));

    Ok(())
  }

  #[allow(clippy::expect_used)]
  #[tokio::test]
  async fn test_update_communities() -> LemmyResult<()> {
    let mut collector = setup_collector();
    let community_id1 = CommunityId(1);
    let community_id2 = CommunityId(2);
    let community_id3 = CommunityId(3);

    let user1_inbox_str = "https://follower1.example.com/inbox";
    let user1_inbox = Url::parse(user1_inbox_str)?;
    let user2_inbox_str = "https://follower2.example.com/inbox";
    let user2_inbox = Url::parse(user2_inbox_str)?;
    let user3_inbox_str = "https://follower3.example.com/inbox";
    let user3_inbox = Url::parse(user3_inbox_str)?;

    collector
      .data_source
      .expect_get_instance_followed_community_inboxes()
      .times(2)
      .returning(move |_, last_fetch| {
        if last_fetch == Utc.timestamp_nanos(0) {
          Ok(vec![
            (community_id1, Url::parse(user1_inbox_str)?.into()),
            (community_id2, Url::parse(user2_inbox_str)?.into()),
          ])
        } else {
          Ok(vec![(community_id3, Url::parse(user3_inbox_str)?.into())])
        }
      });

    // First update
    collector.update_communities().await?;
    assert_eq!(collector.followed_communities.len(), 2);
    assert!(collector.followed_communities[&community_id1].contains(&user1_inbox));
    assert!(collector.followed_communities[&community_id2].contains(&user2_inbox));

    // Simulate time passing
    collector.last_full_communities_fetch =
      Utc::now() - chrono::TimeDelta::try_minutes(3).expect("TimeDelta out of bounds");
    collector.last_incremental_communities_fetch =
      Utc::now() - chrono::TimeDelta::try_minutes(3).expect("TimeDelta out of bounds");

    // Second update (incremental)
    collector.update_communities().await?;
    assert_eq!(collector.followed_communities.len(), 3);
    assert!(collector.followed_communities[&community_id1].contains(&user1_inbox));
    assert!(collector.followed_communities[&community_id3].contains(&user3_inbox));
    assert!(collector.followed_communities[&community_id2].contains(&user2_inbox));

    Ok(())
  }

  #[tokio::test]
  async fn test_get_inbox_urls_no_duplicates() -> LemmyResult<()> {
    let mut collector = setup_collector();
    collector.domain = "example.com".to_string();
    let community_id = CommunityId(1);
    let site_inbox = Url::parse("https://example.com/site_inbox")?;
    let site_inbox_clone = site_inbox.clone();
    let site = Site {
      id: SiteId(1),
      name: "Test Site".to_string(),
      sidebar: None,
      published_at: Utc::now(),
      updated_at: None,
      icon: None,
      banner: None,
      summary: None,
      ap_id: Url::parse("https://example.com/site")?.into(),
      last_refreshed_at: Utc::now(),
      inbox_url: site_inbox.clone().into(),
      private_key: None,
      public_key: "test_key".to_string(),
      instance_id: InstanceId(1),
      content_warning: None,
    };

    collector
      .data_source
      .expect_read_site_from_instance_id()
      .return_once(move |_| Ok(site));

    collector
      .data_source
      .expect_get_instance_followed_community_inboxes()
      .return_once(move |_, _| Ok(vec![(community_id, site_inbox_clone.into())]));

    collector.update_communities().await?;

    let activity = SentActivity {
      id: ActivityId(1),
      ap_id: Url::parse("https://example.com/activities/1")?.into(),
      data: json!({}),
      sensitive: false,
      published_at: Utc::now(),
      send_inboxes: vec![Some(site_inbox.into())],
      send_community_followers_of: Some(community_id),
      send_all_instances: true,
      actor_type: ActorType::Person,
      actor_apub_id: None,
    };

    let result = collector.get_inbox_urls(&activity).await?;
    assert_eq!(result.len(), 1);
    assert!(result.contains(&Url::parse("https://example.com/site_inbox")?));

    Ok(())
  }
}
