use crate::{
  inboxes::CommunityInboxCollector,
  util::{
    get_activity_cached,
    get_actor_cached,
    get_latest_activity_id,
    WORK_FINISHED_RECHECK_DELAY,
  },
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
  newtypes::{ActivityId, InstanceId},
  source::{
    activity::SentActivity,
    federation_queue_state::FederationQueueState,
    instance::{Instance, InstanceForm},
  },
  utils::naive_now,
};
use lemmy_utils::error::LemmyResult;
use std::{
  ops::{Add, Deref},
  time::Duration,
};
use tokio::{sync::mpsc::UnboundedSender, time::sleep};
use tokio_util::sync::CancellationToken;
use tracing::{debug, info, trace, warn};

/// Check whether to save state to db every n sends if there's no failures (during failures state is
/// saved after every attempt). This determines the batch size for loop_batch. After a batch ends
/// and SAVE_STATE_EVERY_TIME has passed, the federation_queue_state is updated in the DB.
static CHECK_SAVE_STATE_EVERY_IT: i64 = 100;
/// Save state to db after this time has passed since the last state (so if the server crashes or is
/// SIGKILLed, less than X seconds of activities are resent)
static SAVE_STATE_EVERY_TIME: Duration = Duration::from_secs(60);

pub(crate) struct InstanceWorker {
  instance: Instance,
  inboxes: CommunityInboxCollector,
  stop: CancellationToken,
  context: Data<LemmyContext>,
  stats_sender: UnboundedSender<(InstanceId, FederationQueueState)>,
  state: FederationQueueState,
  last_state_insert: DateTime<Utc>,
}

impl InstanceWorker {
  pub(crate) async fn init_and_loop(
    instance: Instance,
    context: Data<LemmyContext>,
    stop: CancellationToken,
    stats_sender: UnboundedSender<(InstanceId, FederationQueueState)>,
  ) -> LemmyResult<()> {
    let mut pool = context.pool();
    let state = FederationQueueState::load(&mut pool, instance.id).await?;
    let inboxes = CommunityInboxCollector::new(instance.clone());
    let mut worker = InstanceWorker {
      instance,
      inboxes,
      stop,
      context,
      stats_sender,
      state,
      last_state_insert: Utc.timestamp_nanos(0),
    };
    worker.loop_until_stopped().await
  }
  /// loop fetch new activities from db and send them to the inboxes of the given instances
  /// this worker only returns if (a) there is an internal error or (b) the cancellation token is
  /// cancelled (graceful exit)
  pub(crate) async fn loop_until_stopped(&mut self) -> LemmyResult<()> {
    debug!("Starting federation worker for {}", self.instance.domain);
    let save_state_every = chrono::Duration::from_std(SAVE_STATE_EVERY_TIME).expect("not negative");

    self.inboxes.update_communities(&self.context).await?;
    self.initial_fail_sleep().await?;
    while !self.stop.is_cancelled() {
      self.loop_batch().await?;
      if self.stop.is_cancelled() {
        break;
      }
      if (Utc::now() - self.last_state_insert) > save_state_every {
        self.save_and_send_state().await?;
      }
      self.inboxes.update_communities(&self.context).await?;
    }
    // final update of state in db
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
      tokio::select! {
        () = sleep(remaining) => {},
        () = self.stop.cancelled() => {}
      }
    }
    Ok(())
  }
  /// send out a batch of CHECK_SAVE_STATE_EVERY_IT activities
  async fn loop_batch(&mut self) -> Result<()> {
    let latest_id = get_latest_activity_id(&mut self.context.pool()).await?;
    let mut id = if let Some(id) = self.state.last_successful_id {
      id
    } else {
      // this is the initial creation (instance first seen) of the federation queue for this
      // instance

      // skip all past activities:
      self.state.last_successful_id = Some(latest_id);
      // save here to ensure it's not read as 0 again later if no activities have happened
      self.save_and_send_state().await?;
      latest_id
    };
    if id >= latest_id {
      // no more work to be done, wait before rechecking
      tokio::select! {
        () = sleep(*WORK_FINISHED_RECHECK_DELAY) => {},
        () = self.stop.cancelled() => {}
      }
      return Ok(());
    }
    let mut processed_activities = 0;
    while id < latest_id
      && processed_activities < CHECK_SAVE_STATE_EVERY_IT
      && !self.stop.is_cancelled()
    {
      id = ActivityId(id.0 + 1);
      processed_activities += 1;
      let Some(ele) = get_activity_cached(&mut self.context.pool(), id)
        .await
        .context("failed reading activity from db")?
      else {
        debug!("{}: {:?} does not exist", self.instance.domain, id);
        self.state.last_successful_id = Some(id);
        continue;
      };
      if let Err(e) = self.send_retry_loop(&ele.0, &ele.1).await {
        warn!(
          "sending {} errored internally, skipping activity: {:?}",
          ele.0.ap_id, e
        );
      }
      if self.stop.is_cancelled() {
        return Ok(());
      }
      // send success!
      self.state.last_successful_id = Some(id);
      self.state.last_successful_published_time = Some(ele.0.published);
      self.state.fail_count = 0;
    }
    Ok(())
  }

  // this function will return successfully when (a) send succeeded or (b) worker cancelled
  // and will return an error if an internal error occurred (send errors cause an infinite loop)
  async fn send_retry_loop(
    &mut self,
    activity: &SentActivity,
    object: &SharedInboxActivities,
  ) -> LemmyResult<()> {
    let inbox_urls = self.inboxes.get_inbox_urls(activity, &self.context).await?;
    if inbox_urls.is_empty() {
      trace!("{}: {:?} no inboxes", self.instance.domain, activity.id);
      self.state.last_successful_id = Some(activity.id);
      self.state.last_successful_published_time = Some(activity.published);
      return Ok(());
    }
    let Some(actor_apub_id) = &activity.actor_apub_id else {
      return Ok(()); // activity was inserted before persistent queue was activated
    };
    let actor = get_actor_cached(&mut self.context.pool(), activity.actor_type, actor_apub_id)
      .await
      .context("failed getting actor instance (was it marked deleted / removed?)")?;

    let object = WithContext::new(object.clone(), FEDERATION_CONTEXT.deref().clone());
    let inbox_urls = inbox_urls.into_iter().collect();
    let requests =
      SendActivityTask::prepare(&object, actor.as_ref(), inbox_urls, &self.context).await?;
    for task in requests {
      // usually only one due to shared inbox
      trace!("sending out {}", task);
      while let Err(e) = task.sign_and_send(&self.context).await {
        self.state.fail_count += 1;
        self.state.last_retry = Some(Utc::now());
        let retry_delay: Duration = federate_retry_sleep_duration(self.state.fail_count);
        info!(
          "{}: retrying {:?} attempt {} with delay {retry_delay:.2?}. ({e})",
          self.instance.domain, activity.id, self.state.fail_count
        );
        self.save_and_send_state().await?;
        tokio::select! {
          () = sleep(retry_delay) => {},
          () = self.stop.cancelled() => {
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
        Instance::update(&mut self.context.pool(), self.instance.id, form).await?;
      }
    }
    Ok(())
  }

  async fn save_and_send_state(&mut self) -> Result<()> {
    self.last_state_insert = Utc::now();
    FederationQueueState::upsert(&mut self.context.pool(), &self.state).await?;
    self
      .stats_sender
      .send((self.instance.id, self.state.clone()))?;
    Ok(())
  }
}
