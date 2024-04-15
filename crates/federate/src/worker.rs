use crate::util::{
  get_activity_cached,
  get_actor_cached,
  get_latest_activity_id,
  LEMMY_TEST_FAST_FEDERATION,
  WORK_FINISHED_RECHECK_DELAY,
};
use activitypub_federation::{
  activity_sending::SendActivityTask,
  config::{Data, FederationConfig},
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
  utils::{naive_now, ActualDbPool, DbPool},
};
use lemmy_db_views_actor::structs::CommunityFollowerView;
use once_cell::sync::Lazy;
use reqwest::Url;
use std::{
  collections::{BinaryHeap, HashMap, HashSet},
  ops::{Add, Deref},
  time::Duration,
};
use tokio::{
  sync::mpsc::{self, UnboundedSender},
  time::sleep,
};
use tokio_util::sync::CancellationToken;

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

static CONCURRENT_SENDS: Lazy<i64> = Lazy::new(|| {
  std::env::var("LEMMY_FEDERATION_CONCURRENT_SENDS_PER_INSTANCE")
    .ok()
    .and_then(|s| s.parse().ok())
    .unwrap_or(8)
});
/// Maximum number of successful sends to allow out of order
const MAX_SUCCESSFULS: usize = 1000;

pub(crate) struct InstanceWorker {
  instance: Instance,
  // load site lazily because if an instance is first seen due to being on allowlist,
  // the corresponding row in `site` may not exist yet since that is only added once
  // `fetch_instance_actor_for_object` is called.
  // (this should be unlikely to be relevant outside of the federation tests)
  site_loaded: bool,
  site: Option<Site>,
  followed_communities: HashMap<CommunityId, HashSet<Url>>,
  stop: CancellationToken,
  config: FederationConfig<LemmyContext>,
  stats_sender: UnboundedSender<(String, FederationQueueState)>,
  last_full_communities_fetch: DateTime<Utc>,
  last_incremental_communities_fetch: DateTime<Utc>,
  state: FederationQueueState,
  last_state_insert: DateTime<Utc>,
  pool: ActualDbPool,
}

#[derive(Debug, PartialEq, Eq)]
struct SendSuccessInfo {
  activity_id: ActivityId,
  published: Option<DateTime<Utc>>,
  was_skipped: bool,
}
impl PartialOrd for SendSuccessInfo {
  fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
    other.activity_id.partial_cmp(&self.activity_id)
  }
}
impl Ord for SendSuccessInfo {
  fn cmp(&self, other: &Self) -> std::cmp::Ordering {
    other.activity_id.cmp(&self.activity_id)
  }
}
enum SendActivityResult {
  Success(SendSuccessInfo),
  Failure {
    fail_count: i32,
    // activity_id: ActivityId,
  },
}

impl InstanceWorker {
  pub(crate) async fn init_and_loop(
    instance: Instance,
    config: FederationConfig<LemmyContext>,
    stop: CancellationToken,
    stats_sender: UnboundedSender<(String, FederationQueueState)>,
  ) -> Result<(), anyhow::Error> {
    let pool = config.to_request_data().inner_pool().clone();
    let state = FederationQueueState::load(&mut DbPool::Pool(&pool), instance.id).await?;
    let mut worker = InstanceWorker {
      instance,
      site_loaded: false,
      site: None,
      followed_communities: HashMap::new(),
      stop,
      config,
      stats_sender,
      last_full_communities_fetch: Utc.timestamp_nanos(0),
      last_incremental_communities_fetch: Utc.timestamp_nanos(0),
      state,
      last_state_insert: Utc.timestamp_nanos(0),
      pool,
    };
    worker.loop_until_stopped().await
  }
  /// loop fetch new activities from db and send them to the inboxes of the given instances
  /// this worker only returns if (a) there is an internal error or (b) the cancellation token is cancelled (graceful exit)
  async fn loop_until_stopped(&mut self) -> Result<()> {
    self.initial_fail_sleep().await?;
    let mut latest_id = self.get_latest_id().await?;

    // activities that have been successfully sent but
    // that are not the lowest number and thus can't be written to the database yet
    let mut successfuls = BinaryHeap::<SendSuccessInfo>::new();
    // number of activities that currently have a task spawned to send it
    let mut in_flight: i64 = 0;

    // each HTTP send will report back to this channel concurrently
    let (report_send_result, mut receive_send_result) =
      tokio::sync::mpsc::unbounded_channel::<SendActivityResult>();
    while !self.stop.is_cancelled() {
      // check if we need to wait for a send to finish before sending the next one
      // we wait if (a) the last request failed, only if a request is already in flight (not at the start of the loop)
      // or (b) if we have too many successfuls in memory or (c) if we have too many in flight
      let need_wait_for_event = (in_flight != 0 && self.state.fail_count > 0)
        || successfuls.len() >= MAX_SUCCESSFULS
        || in_flight >= *CONCURRENT_SENDS;
      if need_wait_for_event || receive_send_result.len() > 4 {
        // if len() > 0 then this does not block and allows us to write to db more often
        // if len is 0 then this means we wait for something to change our above conditions,
        // which can only happen by an event sent into the channel
        self
          .handle_send_results(&mut receive_send_result, &mut successfuls, &mut in_flight)
          .await?;
        // handle_send_results does not guarantee that we are now in a condition where we want to  send a new one,
        // so repeat this check until the if no longer applies
        continue;
      } else {
        // send a new activity if there is one
        self.update_communities().await?;
        let next_id = {
          // calculate next id to send based on the last id and the in flight requests
          let last_successful_id = self
            .state
            .last_successful_id
            .map(|e| e.0)
            .expect("set above");
          ActivityId(last_successful_id + (successfuls.len() as i64) + in_flight + 1)
        };
        if next_id > latest_id {
          // lazily fetch latest id only if we have cought up
          latest_id = self.get_latest_id().await?;
          if next_id > latest_id {
            // no more work to be done, wait before rechecking
            tokio::select! {
              () = sleep(*WORK_FINISHED_RECHECK_DELAY) => {},
              () = self.stop.cancelled() => {}
            }
            continue;
          }
        }
        in_flight += 1;
        self
          .spawn_send_if_needed(next_id, report_send_result.clone())
          .await?;
      }
    }
    // final update of state in db on shutdown
    self.save_and_send_state().await?;
    Ok(())
  }

  async fn initial_fail_sleep(&mut self) -> Result<()> {
    // before starting queue, sleep remaining duration if last request failed
    if self.state.fail_count > 0 {
      let last_retry = self
        .state
        .last_retry
        .context("impossible: if fail count set last retry also set")?;
      let elapsed = (Utc::now() - last_retry).to_std()?;
      let required = federate_retry_sleep_duration(self.state.fail_count);
      if elapsed >= required {
        return Ok(());
      }
      let remaining = required - elapsed;
      tracing::debug!(
        "{}: fail-sleeping for {:?} before starting queue",
        self.instance.domain,
        remaining
      );
      tokio::select! {
        () = sleep(remaining) => {},
        () = self.stop.cancelled() => {}
      }
    }
    Ok(())
  }

  /// get newest activity id and set it as last_successful_id if it's the first time this instance is seen
  async fn get_latest_id(&mut self) -> Result<ActivityId> {
    let latest_id = get_latest_activity_id(&mut self.pool()).await?;
    if self.state.last_successful_id.is_none() {
      // this is the initial creation (instance first seen) of the federation queue for this instance
      // skip all past activities:
      self.state.last_successful_id = Some(latest_id);
      // save here to ensure it's not read as 0 again later if no activities have happened
      self.save_and_send_state().await?;
    }
    Ok(latest_id)
  }

  async fn handle_send_results(
    &mut self,
    receive_inbox_result: &mut mpsc::UnboundedReceiver<SendActivityResult>,
    successfuls: &mut BinaryHeap<SendSuccessInfo>,
    in_flight: &mut i64,
  ) -> Result<(), anyhow::Error> {
    let mut force_write = false;
    let mut events = Vec::new();
    // wait for at least one event but if there's multiple handle them all
    receive_inbox_result.recv_many(&mut events, 1000).await;
    for event in events {
      match event {
        SendActivityResult::Success(s) => {
          self.state.fail_count = 0;
          *in_flight -= 1;
          if !s.was_skipped {
            self.mark_instance_alive().await?;
          }
          successfuls.push(s);
        }
        SendActivityResult::Failure { fail_count, .. } => {
          if fail_count > self.state.fail_count {
            // override fail count - if multiple activities are currently sending this value may get conflicting info but that's fine
            self.state.fail_count = fail_count;
            self.state.last_retry = Some(Utc::now());
            force_write = true;
          }
        }
      }
    }
    self
      .pop_successfuls_and_write(successfuls, force_write)
      .await?;
    Ok(())
  }
  async fn mark_instance_alive(&mut self) -> Result<()> {
    // Activity send successful, mark instance as alive if it hasn't been updated in a while.
    let updated = self.instance.updated.unwrap_or(self.instance.published);
    if updated.add(Days::new(1)) < Utc::now() {
      self.instance.updated = Some(Utc::now());

      let form = InstanceForm::builder()
        .domain(self.instance.domain.clone())
        .updated(Some(naive_now()))
        .build();
      Instance::update(&mut self.pool(), self.instance.id, form).await?;
    }
    Ok(())
  }
  /// Checks that sequential activities `last_successful_id + 1`, `last_successful_id + 2` etc have been sent successfully.
  /// In that case updates `last_successful_id` and saves the state to the database if the time since the last save is greater than `SAVE_STATE_EVERY_TIME`.
  async fn pop_successfuls_and_write(
    &mut self,
    successfuls: &mut BinaryHeap<SendSuccessInfo>,
    force_write: bool,
  ) -> Result<()> {
    let Some(mut last_id) = self.state.last_successful_id else {
      tracing::warn!("should be impossible: last successful id is None");
      return Ok(());
    };
    tracing::debug!(
      "last: {:?}, next: {:?}, currently in successfuls: {:?}",
      last_id,
      successfuls.peek(),
      successfuls.iter()
    );
    while successfuls
      .peek()
      .map(|a| &a.activity_id == &ActivityId(last_id.0 + 1))
      .unwrap_or(false)
    {
      let next = successfuls.pop().unwrap();
      last_id = next.activity_id;
      self.state.last_successful_id = Some(next.activity_id);
      self.state.last_successful_published_time = next.published;
    }

    let save_state_every = chrono::Duration::from_std(SAVE_STATE_EVERY_TIME).expect("not negative");
    if force_write || (Utc::now() - self.last_state_insert) > save_state_every {
      self.save_and_send_state().await?;
    }
    Ok(())
  }

  async fn spawn_send_if_needed(
    &mut self,
    activity_id: ActivityId,
    report: UnboundedSender<SendActivityResult>,
  ) -> Result<()> {
    let Some(ele) = get_activity_cached(&mut self.pool(), activity_id)
      .await
      .context("failed reading activity from db")?
    else {
      tracing::debug!("{}: {:?} does not exist", self.instance.domain, activity_id);
      report.send(SendActivityResult::Success(SendSuccessInfo {
        activity_id,
        published: None,
        was_skipped: true,
      }))?;
      return Ok(());
    };
    let activity = &ele.0;
    let inbox_urls = self
      .get_inbox_urls(activity)
      .await
      .context("failed figuring out inbox urls")?;
    if inbox_urls.is_empty() {
      tracing::debug!("{}: {:?} no inboxes", self.instance.domain, activity.id);
      report.send(SendActivityResult::Success(SendSuccessInfo {
        activity_id,
        published: Some(activity.published),
        was_skipped: true,
      }))?;
      return Ok(());
    }
    let initial_fail_count = self.state.fail_count;
    let data = self.config.to_request_data();
    let stop = self.stop.clone();
    let domain = self.instance.domain.clone();
    tokio::spawn(async move {
      let mut report = report;
      if let Err(e) = InstanceWorker::send_retry_loop(
        &ele.0,
        &ele.1,
        inbox_urls,
        &mut report,
        initial_fail_count,
        domain,
        data,
        stop,
      )
      .await
      {
        tracing::warn!(
          "sending {} errored internally, skipping activity: {:?}",
          ele.0.ap_id,
          e
        );
        report
          .send(SendActivityResult::Success(SendSuccessInfo {
            activity_id,
            published: None,
            was_skipped: true,
          }))
          .ok();
      }
    });
    Ok(())
  }

  // this function will return successfully when (a) send succeeded or (b) worker cancelled
  // and will return an error if an internal error occurred (send errors cause an infinite loop)
  async fn send_retry_loop(
    activity: &SentActivity,
    object: &SharedInboxActivities,
    inbox_urls: Vec<Url>,
    report: &mut UnboundedSender<SendActivityResult>,
    initial_fail_count: i32,
    domain: String,
    context: Data<LemmyContext>,
    stop: CancellationToken,
  ) -> Result<()> {
    let pool = &mut context.pool();
    let Some(actor_apub_id) = &activity.actor_apub_id else {
      return Err(anyhow::anyhow!("activity is from before lemmy 0.19"));
    };
    let actor = get_actor_cached(pool, activity.actor_type, actor_apub_id)
      .await
      .context("failed getting actor instance (was it marked deleted / removed?)")?;

    let object = WithContext::new(object.clone(), FEDERATION_CONTEXT.deref().clone());
    let requests = SendActivityTask::prepare(&object, actor.as_ref(), inbox_urls, &context).await?;
    for task in requests {
      // usually only one due to shared inbox
      tracing::debug!("sending out {}", task);
      let mut fail_count = initial_fail_count;
      while let Err(e) = task.sign_and_send(&context).await {
        fail_count += 1;
        report.send(SendActivityResult::Failure {
          fail_count,
          // activity_id: activity.id,
        })?;
        let retry_delay: Duration = federate_retry_sleep_duration(fail_count);
        tracing::info!(
          "{}: retrying {:?} attempt {} with delay {retry_delay:.2?}. ({e})",
          domain,
          activity.id,
          fail_count
        );
        tokio::select! {
          () = sleep(retry_delay) => {},
          () = stop.cancelled() => {
            // save state to db and exit
            // TODO: do we need to report state here to prevent hang on exit?
            return Ok(());
          }
        }
      }
    }
    report.send(SendActivityResult::Success(SendSuccessInfo {
      activity_id: activity.id,
      published: Some(activity.published),
      was_skipped: false,
    }))?;
    Ok(())
  }
  /// get inbox urls of sending the given activity to the given instance
  /// most often this will return 0 values (if instance doesn't care about the activity)
  /// or 1 value (the shared inbox)
  /// > 1 values only happens for non-lemmy software
  async fn get_inbox_urls(&mut self, activity: &SentActivity) -> Result<Vec<Url>> {
    let mut inbox_urls: HashSet<Url> = HashSet::new();

    if activity.send_all_instances {
      if !self.site_loaded {
        self.site = Site::read_from_instance_id(&mut self.pool(), self.instance.id).await?;
        self.site_loaded = true;
      }
      if let Some(site) = &self.site {
        // Nutomic: Most non-lemmy software wont have a site row. That means it cant handle these activities. So handling it like this is fine.
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
    Ok(inbox_urls.into_iter().collect())
  }

  async fn update_communities(&mut self) -> Result<()> {
    if (Utc::now() - self.last_full_communities_fetch) > *FOLLOW_REMOVALS_RECHECK_DELAY {
      tracing::debug!(
        "{}: fetching full list of communities",
        self.instance.domain
      );
      // process removals every hour
      (self.followed_communities, self.last_full_communities_fetch) = self
        .get_communities(self.instance.id, Utc.timestamp_nanos(0))
        .await?;
      self.last_incremental_communities_fetch = self.last_full_communities_fetch;
    }
    if (Utc::now() - self.last_incremental_communities_fetch) > *FOLLOW_ADDITIONS_RECHECK_DELAY {
      // process additions every minute
      let (news, time) = self
        .get_communities(self.instance.id, self.last_incremental_communities_fetch)
        .await?;
      if !news.is_empty() {
        tracing::debug!(
          "{}: fetched {} incremental new followed communities",
          self.instance.domain,
          news.len()
        );
      }
      self.followed_communities.extend(news);
      self.last_incremental_communities_fetch = time;
    }
    Ok(())
  }

  /// get a list of local communities with the remote inboxes on the given instance that cares about them
  async fn get_communities(
    &mut self,
    instance_id: InstanceId,
    last_fetch: DateTime<Utc>,
  ) -> Result<(HashMap<CommunityId, HashSet<Url>>, DateTime<Utc>)> {
    let new_last_fetch =
      Utc::now() - chrono::TimeDelta::try_seconds(10).expect("TimeDelta out of bounds"); // update to time before fetch to ensure overlap. subtract 10s to ensure overlap even if published date is not exact
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
  async fn save_and_send_state(&mut self) -> Result<()> {
    tracing::debug!("{}: saving and sending state", self.instance.domain);
    self.last_state_insert = Utc::now();
    FederationQueueState::upsert(&mut self.pool(), &self.state).await?;
    self
      .stats_sender
      .send((self.instance.domain.clone(), self.state.clone()))?;
    Ok(())
  }

  fn pool(&self) -> DbPool<'_> {
    //self.config.to_request_data()
    DbPool::Pool(&self.pool)
  }
}
