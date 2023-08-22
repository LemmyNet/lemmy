use crate::{
  federation_queue_state::FederationQueueState,
  util::{get_activity_cached, get_actor_cached, get_latest_activity_id, retry_sleep_duration},
};
use activitypub_federation::{activity_sending::SendActivityTask, config::Data};
use anyhow::Result;
use chrono::{DateTime, TimeZone, Utc};
use lemmy_api_common::context::LemmyContext;
use lemmy_apub::activity_lists::SharedInboxActivities;
use lemmy_db_schema::{
  newtypes::{CommunityId, InstanceId},
  source::{activity::SentActivity, instance::Instance, site::Site},
  utils::DbPool,
};
use lemmy_db_views_actor::structs::CommunityFollowerView;
use lemmy_utils::error::LemmyErrorExt2;
use reqwest::Url;
use std::{
  collections::{HashMap, HashSet},
  time::Duration,
};
use tokio::{sync::mpsc::UnboundedSender, time::sleep};
use tokio_util::sync::CancellationToken;
/// save state to db every n sends if there's no failures (otherwise state is saved after every attempt)
static CHECK_SAVE_STATE_EVERY_IT: i64 = 100;
static SAVE_STATE_EVERY_TIME: Duration = Duration::from_secs(10);

pub(crate) struct InstanceWorker {
  instance: Instance,
  site: Option<Site>,
  followed_communities: HashMap<CommunityId, HashSet<Url>>,
  stop: CancellationToken,
  context: Data<LemmyContext>,
  stats_sender: UnboundedSender<FederationQueueState>,
  last_full_communities_fetch: DateTime<Utc>,
  last_incremental_communities_fetch: DateTime<Utc>,
  state: FederationQueueState,
  last_state_insert: DateTime<Utc>,
}

impl InstanceWorker {
  pub(crate) async fn init_and_loop(
    instance: Instance,
    context: Data<LemmyContext>,
    pool: &mut DbPool<'_>, // in theory there's a ref to the pool in context, but i couldn't get that to work wrt lifetimes
    stop: CancellationToken,
    stats_sender: UnboundedSender<FederationQueueState>,
  ) -> Result<(), anyhow::Error> {
    let site = Site::read_from_instance_id(pool, instance.id).await?;
    let state = FederationQueueState::load(pool, &instance.domain).await?;
    let mut worker = InstanceWorker {
      instance,
      site,
      followed_communities: HashMap::new(),
      stop,
      context,
      stats_sender,
      last_full_communities_fetch: Utc.timestamp_nanos(0),
      last_incremental_communities_fetch: Utc.timestamp_nanos(0),
      state,
      last_state_insert: Utc.timestamp_nanos(0),
    };
    worker.loop_until_stopped(pool).await
  }
  /// loop fetch new activities from db and send them to the inboxes of the given instances
  /// this worker only returns if (a) there is an internal error or (b) the cancellation token is cancelled (graceful exit)
  pub(crate) async fn loop_until_stopped(
    &mut self,
    pool: &mut DbPool<'_>,
  ) -> Result<(), anyhow::Error> {
    let save_state_every = chrono::Duration::from_std(SAVE_STATE_EVERY_TIME).expect("not negative");

    self.update_communities(pool).await?;
    self.initial_fail_sleep().await?;
    while !self.stop.is_cancelled() {
      self.loop_batch(pool).await?;
      if self.stop.is_cancelled() {
        break;
      }
      if (Utc::now() - self.last_state_insert) > save_state_every {
        self.save_and_send_state(pool).await?;
      }
      self.update_communities(pool).await?;
    }
    // final update of state in db
    self.save_and_send_state(pool).await?;
    Ok(())
  }

  async fn initial_fail_sleep(&mut self) -> Result<()> {
    // before starting queue, sleep remaining duration if last request failed
    if self.state.fail_count > 0 {
      let elapsed = (Utc::now() - self.state.last_retry).to_std()?;
      let required = retry_sleep_duration(self.state.fail_count);
      if elapsed >= required {
        return Ok(());
      }
      let remaining = required - elapsed;
      tokio::select! {
        () = sleep(remaining) => {},
        () = self.stop.cancelled() => {}
      }
    }
    Ok(())
  }
  async fn loop_batch(&mut self, pool: &mut DbPool<'_>) -> Result<()> {
    let latest_id = get_latest_activity_id(pool).await?;
    let mut id = self.state.last_successful_id;
    if id == latest_id {
      // no more work to be done, wait before rechecking
      tokio::select! {
        () = sleep(Duration::from_secs(10)) => {},
        () = self.stop.cancelled() => {}
      }
      return Ok(());
    }
    let mut processed_activities = 0;
    while id < latest_id
      && processed_activities < CHECK_SAVE_STATE_EVERY_IT
      && !self.stop.is_cancelled()
    {
      id += 1;
      processed_activities += 1;
      let Some(ele) = get_activity_cached(pool, id).await? else {
        self.state.last_successful_id = id;
        continue;
      };
      self.send_retry_loop(pool, &ele.0, &ele.1).await?;
      if self.stop.is_cancelled() {
        return Ok(());
      }
      // send success!
      self.state.last_successful_id = id;
      self.state.fail_count = 0;
    }
    Ok(())
  }

  /** this function will only return if (a) send succeeded or (b) worker cancelled */
  async fn send_retry_loop(
    &mut self,
    pool: &mut DbPool<'_>,
    activity: &SentActivity,
    object: &SharedInboxActivities,
  ) -> Result<()> {
    let inbox_urls = self.get_inbox_urls(activity);
    if inbox_urls.is_empty() {
      self.state.last_successful_id = activity.id;
      return Ok(());
    }
    let Some(actor_apub_id) = &activity.actor_apub_id else {
      return Ok(()); // activity was inserted before persistent queue was activated
    };
    let actor = get_actor_cached(pool, activity.actor_type, actor_apub_id).await?;

    let inbox_urls = inbox_urls.into_iter().collect();
    let requests = SendActivityTask::prepare(object, actor.as_ref(), inbox_urls, &self.context)
      .await
      .into_anyhow()?;
    for task in requests {
      // usually only one due to shared inbox
      tracing::info!("sending out {}", task);
      while let Err(e) = task.sign_and_send(&self.context).await {
        self.state.fail_count += 1;
        self.state.last_retry = Utc::now();
        let retry_delay: Duration = retry_sleep_duration(self.state.fail_count);
        tracing::info!(
          "{}: retrying {} attempt {} with delay {retry_delay:.2?}. ({e})",
          self.instance.domain,
          activity.id,
          self.state.fail_count
        );
        self.save_and_send_state(pool).await?;
        tokio::select! {
          () = sleep(retry_delay) => {},
          () = self.stop.cancelled() => {
            // save state to db and exit
            return Ok(());
          }
        }
      }
    }
    Ok(())
  }

  /// get inbox urls of sending the given activity to the given instance
  /// most often this will return 0 values (if instance doesn't care about the activity)
  /// or 1 value (the shared inbox)
  /// > 1 values only happens for non-lemmy software
  fn get_inbox_urls(&self, activity: &SentActivity) -> HashSet<Url> {
    let mut inbox_urls: HashSet<Url> = HashSet::new();

    if activity.send_all_instances {
      if let Some(site) = &self.site {
        // Nutomic: Most non-lemmy software wont have a site row. That means it cant handle these activities. So handling it like this is fine.
        inbox_urls.insert(site.inbox_url.inner().clone());
      }
    }
    if let Some(t) = &activity.send_community_followers_of {
      if let Some(urls) = self.followed_communities.get(t) {
        inbox_urls.extend(urls.iter().map(std::clone::Clone::clone));
      }
    }
    inbox_urls.extend(
      activity
        .send_inboxes
        .iter()
        .filter_map(std::option::Option::as_ref)
        .filter_map(|u| (u.domain() == Some(&self.instance.domain)).then(|| u.inner().clone())),
    );
    inbox_urls
  }

  async fn update_communities(&mut self, pool: &mut DbPool<'_>) -> Result<()> {
    if (Utc::now() - self.last_full_communities_fetch) > chrono::Duration::seconds(600) {
      // process removals every 5min
      (self.followed_communities, self.last_full_communities_fetch) = self
        .get_communities(pool, self.instance.id, self.last_full_communities_fetch)
        .await?;
      self.last_incremental_communities_fetch = self.last_full_communities_fetch;
    }
    if (Utc::now() - self.last_incremental_communities_fetch) > chrono::Duration::seconds(60) {
      let (news, time) = self
        .get_communities(
          pool,
          self.instance.id,
          self.last_incremental_communities_fetch,
        )
        .await?;
      // process additions every 10s
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
    let new_last_fetch = Utc::now(); // update to time before fetch to ensure overlap
    Ok((
      CommunityFollowerView::get_instance_followed_community_inboxes(pool, instance_id, last_fetch)
        .await?
        .into_iter()
        .fold(HashMap::new(), |mut map, (c, u)| {
          map.entry(c).or_insert_with(HashSet::new).insert(u.into());
          map
        }),
      new_last_fetch,
    ))
  }
  async fn save_and_send_state(&mut self, pool: &mut DbPool<'_>) -> Result<()> {
    self.last_state_insert = Utc::now();
    FederationQueueState::upsert(pool, &self.state).await?;
    self.stats_sender.send(self.state.clone())?;
    Ok(())
  }
}
