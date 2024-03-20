use crate::util::{
  get_activity_cached,
  get_actor_cached,
  get_latest_activity_id,
  LEMMY_TEST_FAST_FEDERATION,
  WORK_FINISHED_RECHECK_DELAY,
};
use activitypub_federation::{
  activity_sending::SendActivityTask,
  config::Data,
  protocol::context::WithContext,
};
use anyhow::{Context, Result};
use chrono::{DateTime, Days, TimeZone, Utc};
use lemmy_api_common::{context::LemmyContext, federate_retry_sleep_duration};
use lemmy_apub::{activity_lists::SharedInboxActivities, FEDERATION_CONTEXT};
use lemmy_db_schema::{
  newtypes::{ActivityId, CommunityId, InstanceId},
  source::{
    activity::SentActivity,
    federation_queue_state::FederationQueueState,
    instance::{Instance, InstanceForm},
    site::Site,
  },
  utils::{naive_now, DbPool},
};
use lemmy_db_views_actor::structs::CommunityFollowerView;
use once_cell::sync::Lazy;
use reqwest::Url;
use std::{
  collections::{HashMap, HashSet},
  ops::{Add, Deref},
  sync::{
    atomic::{AtomicU8, Ordering},
    Arc,
    Mutex,
  },
  time::Duration,
};
use tokio::{spawn, sync::mpsc::UnboundedSender, time::sleep};
use tokio_util::sync::CancellationToken;

/// Check whether to save state to db every n sends if there's no failures (during failures state is saved after every attempt)
/// This determines the batch size for loop_batch. After a batch ends and SAVE_STATE_EVERY_TIME has passed, the federation_queue_state is updated in the DB.
static CHECK_SAVE_STATE_EVERY_IT: i64 = 100;
/// Save state to db after this time has passed since the last state (so if the server crashes or is SIGKILLed, less than X seconds of activities are resent)
static SAVE_STATE_EVERY_TIME: Duration = Duration::from_secs(60);
/// interval with which new additions to community_followers are queried.
///
/// The first time some user on an instance follows a specific remote community (or, more precisely: the first time a (followed_community_id, follower_inbox_url) tuple appears),
/// this delay limits the maximum time until the follow actually results in activities from that community id being sent to that inbox url.
/// This delay currently needs to not be too small because the DB load is currently fairly high because of the current structure of storing inboxes for every person, not having a separate list of shared_inboxes, and the architecture of having every instance queue be fully separate.
/// (see https://github.com/LemmyNet/lemmy/issues/3958)
static FOLLOW_ADDITIONS_RECHECK_DELAY: Lazy<chrono::TimeDelta> = Lazy::new(|| {
  if *LEMMY_TEST_FAST_FEDERATION {
    chrono::TimeDelta::try_seconds(1).expect("TimeDelta out of bounds")
  } else {
    chrono::TimeDelta::try_minutes(2).expect("TimeDelta out of bounds")
  }
});
/// The same as FOLLOW_ADDITIONS_RECHECK_DELAY, but triggering when the last person on an instance unfollows a specific remote community.
/// This is expected to happen pretty rarely and updating it in a timely manner is not too important.
static FOLLOW_REMOVALS_RECHECK_DELAY: Lazy<chrono::TimeDelta> =
  Lazy::new(|| chrono::TimeDelta::try_hours(1).expect("TimeDelta out of bounds"));

const MAX_INFLIGHT_REQUESTS: u8 = 5;

#[derive(Clone)]
pub(crate) struct InstanceWorker {
  instance: Instance,
  context: Data<LemmyContext>,
  data: Arc<Mutex<InstanceWorkerData>>,
  stats_sender: UnboundedSender<(String, FederationQueueState)>,
}
impl InstanceWorker {
  fn is_cancelled(&self) -> bool {
    self.data.lock().unwrap().stop.is_cancelled()
  }
  async fn cancelled(&self) {
    let stop = {
      let lock = self.data.lock().unwrap();
      lock.stop.clone()
    };
    stop.cancelled().await
  }
  fn state(&self) -> FederationQueueState {
    self.data.lock().unwrap().state.clone()
  }
  fn last_state_insert(&self) -> DateTime<Utc> {
    self.data.lock().unwrap().last_state_insert.clone()
  }
  async fn site(&self, pool: &mut DbPool<'_>) -> Option<Site> {
    let site_loaded = {
      let lock = self.data.lock().unwrap();
      lock.site_loaded
    };
    if !site_loaded {
      let site = Site::read_from_instance_id(pool, self.instance.id)
        .await
        .unwrap();
      let mut lock = self.data.lock().unwrap();
      lock.site = site;
      lock.site_loaded = true;
    }
    self.data.lock().unwrap().site.clone()
  }
  async fn update_communities(&self, pool: &mut DbPool<'_>) {
    let mut communities = {
      let lock = self.data.lock().unwrap();
      lock.communities.clone()
    };
    communities
      .update_communities(self.instance.id, pool)
      .await
      .unwrap();
    self.data.lock().unwrap().communities = communities;
  }
}

struct InstanceWorkerData {
  // load site lazily because if an instance is first seen due to being on allowlist,
  // the corresponding row in `site` may not exist yet since that is only added once
  // `fetch_instance_actor_for_object` is called.
  // (this should be unlikely to be relevant outside of the federation tests)
  site_loaded: bool,
  site: Option<Site>,
  stop: CancellationToken,
  state: FederationQueueState,
  last_state_insert: DateTime<Utc>,
  communities: InstanceWorkerCommunities,
}

#[derive(Clone)]
struct InstanceWorkerCommunities {
  followed_communities: HashMap<CommunityId, HashSet<Url>>,
  last_full_communities_fetch: DateTime<Utc>,
  last_incremental_communities_fetch: DateTime<Utc>,
}

impl InstanceWorker {
  pub(crate) async fn init_and_loop(
    instance: Instance,
    context: Data<LemmyContext>,
    stop: CancellationToken,
    stats_sender: UnboundedSender<(String, FederationQueueState)>,
  ) -> Result<(), anyhow::Error> {
    let state = FederationQueueState::load(&mut context.pool(), instance.id).await?;
    let mut worker = InstanceWorker {
      instance,
      context: context.reset_request_count(),
      stats_sender,
      data: Arc::new(Mutex::new(InstanceWorkerData {
        site_loaded: false,
        site: None,
        stop,
        state,
        last_state_insert: Utc.timestamp_nanos(0),
        communities: InstanceWorkerCommunities {
          followed_communities: HashMap::new(),
          last_full_communities_fetch: Utc.timestamp_nanos(0),
          last_incremental_communities_fetch: Utc.timestamp_nanos(0),
        },
      })),
    };
    worker.loop_until_stopped(&context).await
  }
  /// loop fetch new activities from db and send them to the inboxes of the given instances
  /// this worker only returns if (a) there is an internal error or (b) the cancellation token is cancelled (graceful exit)
  pub(crate) async fn loop_until_stopped(
    &mut self,
    context: &LemmyContext,
  ) -> Result<(), anyhow::Error> {
    let pool = &mut context.pool();
    let save_state_every = chrono::Duration::from_std(SAVE_STATE_EVERY_TIME).expect("not negative");

    self.update_communities(pool).await;
    self.initial_fail_sleep().await?;
    while !self.is_cancelled() {
      self.loop_batch(&context).await?;
      if self.is_cancelled() {
        break;
      }
      if (Utc::now() - self.last_state_insert()) > save_state_every {
        self.save_and_send_state(pool).await?;
      }
      self.update_communities(pool).await;
    }
    // final update of state in db
    self.save_and_send_state(pool).await?;
    Ok(())
  }

  async fn initial_fail_sleep(&mut self) -> Result<()> {
    // before starting queue, sleep remaining duration if last request failed
    if self.state().fail_count > 0 {
      let last_retry = self
        .state()
        .last_retry
        .context("impossible: if fail count set last retry also set")?;
      let elapsed = (Utc::now() - last_retry).to_std()?;
      let required = federate_retry_sleep_duration(self.state().fail_count);
      if elapsed >= required {
        return Ok(());
      }
      let remaining = required - elapsed;
      tokio::select! {
        () = sleep(remaining) => {},
        () = self.cancelled() => {}
      }
    }
    Ok(())
  }
  /// send out a batch of CHECK_SAVE_STATE_EVERY_IT activities
  async fn loop_batch(&mut self, context: &LemmyContext) -> Result<()> {
    let latest_id = get_latest_activity_id(&mut context.pool()).await?;
    let mut id = if let Some(id) = self.state().last_successful_id {
      id
    } else {
      // this is the initial creation (instance first seen) of the federation queue for this instance
      // skip all past activities:
      self.state().last_successful_id = Some(latest_id);
      // save here to ensure it's not read as 0 again later if no activities have happened
      self.save_and_send_state(&mut context.pool()).await?;
      latest_id
    };
    if id >= latest_id {
      if latest_id.0 != 0 {
        dbg!("work done", id, latest_id);
      }
      // no more work to be done, wait before rechecking
      tokio::select! {
        () = sleep(*WORK_FINISHED_RECHECK_DELAY) => {},
        () = self.cancelled() => {}
      }
      return Ok(());
    }
    // TODO: somehow its not reaching here and not sending anything
    dbg!("loop 1");
    let mut processed_activities = 0;
    let inflight_requests = Arc::new(AtomicU8::new(0));
    dbg!("loop 2");
    while id < latest_id && processed_activities < CHECK_SAVE_STATE_EVERY_IT && !self.is_cancelled()
    {
      dbg!("loop 3");
      while dbg!(inflight_requests.load(Ordering::Relaxed)) >= MAX_INFLIGHT_REQUESTS {
        dbg!("sleep");
        sleep(Duration::from_millis(100)).await;
      }
      id = ActivityId(id.0 + 1);
      processed_activities += 1;
      dbg!("even before sending activity", id);
      let Some(ele) = get_activity_cached(&mut context.pool(), id)
        .await
        .context("failed reading activity from db")?
      else {
        dbg!("first send, set last successful to current");
        tracing::debug!("{}: {:?} does not exist", self.instance.domain, id);
        self.state().last_successful_id = Some(id);
        continue;
      };
      let context = context.clone();
      let inflight_requests = inflight_requests.clone();
      let mut self_ = self.clone();
      dbg!(&ele);
      dbg!("before sending activity", ele.0.id);
      spawn(async move {
        dbg!("during sending activity", ele.0.id);
        dbg!(inflight_requests.fetch_add(1, Ordering::Relaxed) + 1);
        if let Err(e) = self_
          .send_retry_loop(&mut context.pool(), &ele.0, &ele.1)
          .await
        {
          tracing::warn!(
            "sending {} errored internally, skipping activity: {:?}",
            ele.0.ap_id,
            e
          );
        }
        inflight_requests.fetch_sub(1, Ordering::Relaxed);
        // send success!
        self_.state().last_successful_id = Some(id);
        self_.state().last_successful_published_time = Some(ele.0.published);
        self_.state().fail_count = 0;
      });
      if self.is_cancelled() {
        return Ok(());
      }
    }
    Ok(())
  }

  // this function will return successfully when (a) send succeeded or (b) worker cancelled
  // and will return an error if an internal error occurred (send errors cause an infinite loop)
  async fn send_retry_loop(
    &mut self,
    pool: &mut DbPool<'_>,
    activity: &SentActivity,
    object: &SharedInboxActivities,
  ) -> Result<()> {
    let inbox_urls = self
      .get_inbox_urls(pool, activity)
      .await
      .context("failed figuring out inbox urls")?;
    if inbox_urls.is_empty() {
      tracing::debug!("{}: {:?} no inboxes", self.instance.domain, activity.id);
      self.state().last_successful_id = Some(activity.id);
      self.state().last_successful_published_time = Some(activity.published);
      return Ok(());
    }
    let Some(actor_apub_id) = &activity.actor_apub_id else {
      return Ok(()); // activity was inserted before persistent queue was activated
    };
    let actor = get_actor_cached(pool, activity.actor_type, actor_apub_id)
      .await
      .context("failed getting actor instance (was it marked deleted / removed?)")?;

    let object = WithContext::new(object.clone(), FEDERATION_CONTEXT.deref().clone());
    let inbox_urls = inbox_urls.into_iter().collect();
    let requests =
      SendActivityTask::prepare(&object, actor.as_ref(), inbox_urls, &self.context).await?;
    for task in requests {
      // usually only one due to shared inbox
      tracing::debug!("sending out {}", task);
      while let Err(e) = task.sign_and_send(&self.context).await {
        self.state().fail_count += 1;
        self.state().last_retry = Some(Utc::now());
        let retry_delay: Duration = federate_retry_sleep_duration(self.state().fail_count);
        tracing::info!(
          "{}: retrying {:?} attempt {} with delay {retry_delay:.2?}. ({e})",
          self.instance.domain,
          activity.id,
          self.state().fail_count
        );
        self.save_and_send_state(pool).await?;
        tokio::select! {
          () = sleep(retry_delay) => {},
          () = self.cancelled() => {
            // save state to db and exit
            return Ok(());
          }
        }
      }

      // Activity send successful, mark instance as alive if it hasn't been updated in a while.
      let updated = self.instance.updated.unwrap_or(self.instance.published);
      if updated.add(Days::new(1)) < Utc::now() {
        self.instance.updated = Some(Utc::now());

        let form = InstanceForm::builder()
          .domain(self.instance.domain.clone())
          .updated(Some(naive_now()))
          .build();
        Instance::update(pool, self.instance.id, form).await?;
      }
    }
    Ok(())
  }

  /// get inbox urls of sending the given activity to the given instance
  /// most often this will return 0 values (if instance doesn't care about the activity)
  /// or 1 value (the shared inbox)
  /// > 1 values only happens for non-lemmy software
  async fn get_inbox_urls(
    &mut self,
    pool: &mut DbPool<'_>,
    activity: &SentActivity,
  ) -> Result<HashSet<Url>> {
    let mut inbox_urls: HashSet<Url> = HashSet::new();

    if activity.send_all_instances {
      if let Some(site) = &self.site(pool).await {
        // Nutomic: Most non-lemmy software wont have a site row. That means it cant handle these activities. So handling it like this is fine.
        inbox_urls.insert(site.inbox_url.inner().clone());
      }
    }
    if let Some(t) = &activity.send_community_followers_of {
      if let Some(urls) = self
        .data
        .lock()
        .unwrap()
        .communities
        .followed_communities
        .get(t)
      {
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

  async fn save_and_send_state(&mut self, pool: &mut DbPool<'_>) -> Result<()> {
    {
      self.data.lock().unwrap().last_state_insert = Utc::now();
    }
    FederationQueueState::upsert(pool, &self.state()).await?;
    self
      .stats_sender
      .send((self.instance.domain.clone(), self.state().clone()))?;
    Ok(())
  }
}

impl InstanceWorkerCommunities {
  async fn update_communities(
    &mut self,
    instance_id: InstanceId,
    pool: &mut DbPool<'_>,
  ) -> Result<()> {
    if (Utc::now() - self.last_full_communities_fetch) > *FOLLOW_REMOVALS_RECHECK_DELAY {
      // process removals every hour
      (self.followed_communities, self.last_full_communities_fetch) = self
        .get_communities(pool, instance_id, Utc.timestamp_nanos(0))
        .await?;
      self.last_incremental_communities_fetch = self.last_full_communities_fetch;
    }
    if (Utc::now() - self.last_incremental_communities_fetch) > *FOLLOW_ADDITIONS_RECHECK_DELAY {
      // process additions every minute
      let (news, time) = self
        .get_communities(pool, instance_id, self.last_incremental_communities_fetch)
        .await?;
      self.followed_communities.extend(news);
      self.last_incremental_communities_fetch = time;
    }
    Ok(())
  }

  /// get a list of local communities with the remote inboxes on the given instance that cares about them
  async fn get_communities(
    &mut self,
    pool: &mut DbPool<'_>,
    instance_id: InstanceId,
    last_fetch: DateTime<Utc>,
  ) -> Result<(HashMap<CommunityId, HashSet<Url>>, DateTime<Utc>)> {
    let new_last_fetch =
      Utc::now() - chrono::TimeDelta::try_seconds(10).expect("TimeDelta out of bounds"); // update to time before fetch to ensure overlap. subtract 10s to ensure overlap even if published date is not exact
    Ok((
      CommunityFollowerView::get_instance_followed_community_inboxes(pool, instance_id, last_fetch)
        .await?
        .into_iter()
        .fold(HashMap::new(), |mut map, (c, u)| {
          map.entry(c).or_default().insert(u.into());
          map
        }),
      new_last_fetch,
    ))
  }
}
