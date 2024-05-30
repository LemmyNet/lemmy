use crate::{
  inboxes::CommunityInboxCollector,
  util::{
    get_activity_cached,
    get_actor_cached,
    get_latest_activity_id,
    FederationQueueStateWithDomain,
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
  newtypes::ActivityId,
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
#[cfg(debug_assertions)]
static SAVE_STATE_EVERY_TIME: chrono::Duration = chrono::Duration::seconds(1);
#[cfg(not(debug_assertions))]
static SAVE_STATE_EVERY_TIME: chrono::Duration = chrono::Duration::seconds(60);

pub(crate) struct InstanceWorker {
  target: Instance,
  inboxes: CommunityInboxCollector,
  stop: CancellationToken,
  context: Data<LemmyContext>,
  stats_sender: UnboundedSender<FederationQueueStateWithDomain>,
  state: FederationQueueState,
  last_state_insert: DateTime<Utc>,
}

impl InstanceWorker {
  pub(crate) async fn init_and_loop(
    instance: Instance,
    context: Data<LemmyContext>,
    stop: CancellationToken,
    stats_sender: UnboundedSender<FederationQueueStateWithDomain>,
  ) -> LemmyResult<()> {
    let mut pool = context.pool();
    let state = FederationQueueState::load(&mut pool, instance.id).await?;
    let inboxes = CommunityInboxCollector::new(instance.clone());
    let mut worker = InstanceWorker {
      target: instance,
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
    debug!("Starting federation worker for {}", self.target.domain);
    self.inboxes.update_communities(&self.context).await?;
    self.initial_fail_sleep().await?;
    while !self.stop.is_cancelled() {
      self.loop_batch().await?;
      if self.stop.is_cancelled() {
        break;
      }
      if (Utc::now() - self.last_state_insert) > SAVE_STATE_EVERY_TIME {
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
        () = sleep(WORK_FINISHED_RECHECK_DELAY) => {},
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
        debug!("{}: {:?} does not exist", self.target.domain, id);
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
    println!("send retry loop {:?}", activity.id);
    let inbox_urls = self.inboxes.get_inbox_urls(activity, &self.context).await?;
    if inbox_urls.is_empty() {
      trace!("{}: {:?} no inboxes", self.target.domain, activity.id);
      self.state.last_successful_id = Some(activity.id);
      self.state.last_successful_published_time = Some(activity.published);
      return Ok(());
    }
    let actor = get_actor_cached(
      &mut self.context.pool(),
      activity.actor_type,
      &activity.actor_apub_id,
    )
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
          self.target.domain, activity.id, self.state.fail_count
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
      let updated = self.target.updated.unwrap_or(self.target.published);
      dbg!(&updated);
      if updated.add(Days::new(1)) < Utc::now() {
        self.target.updated = Some(Utc::now());

        let form = InstanceForm::builder()
          .domain(self.target.domain.clone())
          .updated(Some(naive_now()))
          .build();
        Instance::update(&mut self.context.pool(), self.target.id, form).await?;
      }
    }
    Ok(())
  }

  async fn save_and_send_state(&mut self) -> Result<()> {
    self.last_state_insert = Utc::now();
    FederationQueueState::upsert(&mut self.context.pool(), &self.state).await?;
    self.stats_sender.send(FederationQueueStateWithDomain {
      state: self.state.clone(),
      domain: self.target.domain.clone(),
    })?;
    Ok(())
  }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
#[allow(clippy::indexing_slicing)]
mod test {

  use super::*;
  use activitypub_federation::http_signatures::generate_actor_keypair;
  use actix_web::{rt::System, web, App, HttpResponse, HttpServer};
  use lemmy_api_common::utils::{generate_inbox_url, generate_shared_inbox_url};
  use lemmy_db_schema::{
    newtypes::DbUrl,
    source::{
      activity::{ActorType, SentActivityForm},
      person::{Person, PersonInsertForm},
    },
    traits::Crud,
  };
  use reqwest::StatusCode;
  use serde_json::Value;
  use serial_test::serial;
  use std::{fs::File, io::BufReader};
  use tokio::{
    select,
    spawn,
    sync::mpsc::{error::TryRecvError, unbounded_channel, UnboundedReceiver},
  };
  use url::Url;

  struct Data {
    context: activitypub_federation::config::Data<LemmyContext>,
    instance: Instance,
    person: Person,
    stats_receiver: UnboundedReceiver<FederationQueueStateWithDomain>,
    inbox_receiver: UnboundedReceiver<String>,
    cancel: CancellationToken,
  }

  impl Data {
    async fn init() -> LemmyResult<Self> {
      let context = LemmyContext::init_test_context().await;
      let instance = Instance::read_or_create(&mut context.pool(), "localhost".to_string()).await?;

      let actor_keypair = generate_actor_keypair()?;
      let actor_id: DbUrl = Url::parse("http://local.com/u/alice")?.into();
      let person_form = PersonInsertForm::builder()
        .name("alice".to_string())
        .actor_id(Some(actor_id.clone()))
        .private_key(Some(actor_keypair.private_key))
        .public_key(actor_keypair.public_key)
        .inbox_url(Some(generate_inbox_url(&actor_id)?))
        .shared_inbox_url(Some(generate_shared_inbox_url(context.settings())?))
        .instance_id(instance.id)
        .build();
      let person = Person::create(&mut context.pool(), &person_form).await?;

      let cancel = CancellationToken::new();
      let (stats_sender, stats_receiver) = unbounded_channel();
      let (inbox_sender, inbox_receiver) = unbounded_channel();

      // listen for received activities in background
      let cancel_ = cancel.clone();
      std::thread::spawn(move || System::new().block_on(listen_activities(inbox_sender, cancel_)));

      spawn(InstanceWorker::init_and_loop(
        instance.clone(),
        context.reset_request_count(),
        cancel.clone(),
        stats_sender,
      ));
      // wait for startup
      sleep(WORK_FINISHED_RECHECK_DELAY).await;

      Ok(Self {
        context,
        instance,
        person,
        stats_receiver,
        inbox_receiver,
        cancel,
      })
    }

    async fn cleanup(&self) -> LemmyResult<()> {
      self.cancel.cancel();
      sleep(WORK_FINISHED_RECHECK_DELAY).await;
      Instance::delete_all(&mut self.context.pool()).await?;
      Person::delete(&mut self.context.pool(), self.person.id).await?;
      Ok(())
    }
  }

  #[tokio::test]
  #[serial]
  async fn test_stats() -> LemmyResult<()> {
    let mut data = Data::init().await?;

    // first receive at startup
    let rcv = data.stats_receiver.recv().await.unwrap();
    assert_eq!(data.instance.id, rcv.state.instance_id);
    assert_eq!(Some(ActivityId(0)), rcv.state.last_successful_id);

    let sent = send_activity(data.person.actor_id.clone(), &data.context).await?;

    // receive for successfully sent activity
    let inbox_rcv = data.inbox_receiver.recv().await.unwrap();
    let parsed_activity = serde_json::from_str::<WithContext<Value>>(&inbox_rcv)?;
    assert_eq!(&sent.data, parsed_activity.inner());

    let rcv = data.stats_receiver.recv().await.unwrap();
    assert_eq!(data.instance.id, rcv.state.instance_id);
    assert_eq!(Some(sent.id), rcv.state.last_successful_id);

    data.cleanup().await?;

    // it also sends state on shutdown
    let rcv = data.stats_receiver.try_recv();
    assert!(rcv.is_ok());

    // nothing further received
    let rcv = data.stats_receiver.try_recv();
    assert_eq!(Some(TryRecvError::Disconnected), rcv.err());
    let inbox_rcv = data.inbox_receiver.try_recv();
    assert_eq!(Some(TryRecvError::Empty), inbox_rcv.err());

    Ok(())
  }

  #[tokio::test]
  #[serial]
  async fn test_update_instance() -> LemmyResult<()> {
    let mut data = Data::init().await?;

    let published = DateTime::from_timestamp_nanos(0);
    let form = InstanceForm::builder()
      .domain(data.instance.domain.clone())
      .published(Some(published))
      .updated(None)
      .build();
    Instance::update(&mut data.context.pool(), data.instance.id, form).await?;

    send_activity(data.person.actor_id.clone(), &data.context).await?;
    data.inbox_receiver.recv().await.unwrap();

    let instance =
      Instance::read_or_create(&mut data.context.pool(), data.instance.domain.clone()).await?;

    assert!(instance.updated.is_some());

    data.cleanup().await?;

    Ok(())
  }

  async fn listen_activities(
    inbox_sender: UnboundedSender<String>,
    cancel: CancellationToken,
  ) -> LemmyResult<()> {
    let run = HttpServer::new(move || {
      App::new()
        .app_data(actix_web::web::Data::new(inbox_sender.clone()))
        .route(
          "/inbox",
          web::post().to(
            |inbox_sender: actix_web::web::Data<UnboundedSender<String>>, body: String| async move {
              inbox_sender.send(body.clone()).unwrap();
              HttpResponse::new(StatusCode::OK)
            },
          ),
        )
    })
    .bind(("127.0.0.1", 8085))?
    .run();
    select! {
      _ = run => {},
      _ = cancel.cancelled() => {
    }
    }
    Ok(())
  }

  async fn send_activity(actor_id: DbUrl, context: &LemmyContext) -> LemmyResult<SentActivity> {
    // create outgoing activity
    let file = File::open("../apub/assets/lemmy/activities/voting/like_note.json")?;
    let reader = BufReader::new(file);
    let form = SentActivityForm {
      ap_id: Url::parse("http://local.com/activity/1")?.into(),
      data: serde_json::from_reader(reader)?,
      sensitive: false,
      send_inboxes: vec![Some(Url::parse("http://localhost:8085/inbox")?.into())],
      send_all_instances: false,
      send_community_followers_of: None,
      actor_type: ActorType::Person,
      actor_apub_id: actor_id,
    };
    let sent = SentActivity::create(&mut context.pool(), form).await?;

    sleep(WORK_FINISHED_RECHECK_DELAY * 2).await;

    Ok(sent)
  }
}
