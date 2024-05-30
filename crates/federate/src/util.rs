use anyhow::{anyhow, Context, Result};
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use lemmy_api_common::lemmy_utils::CACHE_DURATION_FEDERATION;
use lemmy_apub::{
  activity_lists::SharedInboxActivities,
  fetcher::{site_or_community_or_user::SiteOrCommunityOrUser, user_or_community::UserOrCommunity},
};
use lemmy_db_schema::{
  newtypes::ActivityId,
  source::{
    activity::{ActorType, SentActivity},
    community::Community,
    federation_queue_state::FederationQueueState,
    person::Person,
    site::Site,
  },
  traits::ApubActor,
  utils::{get_conn, DbPool},
};
use moka::future::Cache;
use once_cell::sync::Lazy;
use reqwest::Url;
use serde_json::Value;
use std::{fmt::Debug, future::Future, pin::Pin, sync::Arc, time::Duration};
use tokio::{task::JoinHandle, time::sleep};
use tokio_util::sync::CancellationToken;

/// Decrease the delays of the federation queue.
/// Should only be used for federation tests since it significantly increases CPU and DB load of the
/// federation queue.
pub(crate) static LEMMY_TEST_FAST_FEDERATION: Lazy<bool> = Lazy::new(|| {
  std::env::var("LEMMY_TEST_FAST_FEDERATION")
    .map(|s| !s.is_empty())
    .unwrap_or(false)
});

/// Recheck for new federation work every n seconds.
///
/// When the queue is processed faster than new activities are added and it reaches the current time
/// with an empty batch, this is the delay the queue waits before it checks if new activities have
/// been added to the sent_activities table. This delay is only applied if no federated activity
/// happens during sending activities of the last batch.
pub(crate) static WORK_FINISHED_RECHECK_DELAY: Lazy<Duration> = Lazy::new(|| {
  if *LEMMY_TEST_FAST_FEDERATION {
    Duration::from_millis(100)
  } else {
    Duration::from_secs(30)
  }
});

/// A task that will be run in an infinite loop, unless it is cancelled.
/// If the task exits without being cancelled, an error will be logged and the task will be
/// restarted.
pub struct CancellableTask {
  f: Pin<Box<dyn Future<Output = Result<(), anyhow::Error>> + Send + 'static>>,
}

impl CancellableTask {
  /// spawn a task but with graceful shutdown
  pub fn spawn<F, R>(
    timeout: Duration,
    task: impl Fn(CancellationToken) -> F + Send + 'static,
  ) -> CancellableTask
  where
    F: Future<Output = R> + Send + 'static,
    R: Send + Debug + 'static,
  {
    let stop = CancellationToken::new();
    let stop2 = stop.clone();
    let task: JoinHandle<()> = tokio::spawn(async move {
      loop {
        let res = task(stop2.clone()).await;
        if stop2.is_cancelled() {
          return;
        } else {
          tracing::warn!("task exited, restarting: {res:?}");
        }
      }
    });
    let abort = task.abort_handle();
    CancellableTask {
      f: Box::pin(async move {
        stop.cancel();
        tokio::select! {
            r = task => {
              r.context("CancellableTask failed to cancel cleanly, returned error")?;
              Ok(())
            },
            _ = sleep(timeout) => {
                abort.abort();
                Err(anyhow!("CancellableTask aborted due to shutdown timeout"))
            }
        }
      }),
    }
  }

  /// cancel the cancel signal, wait for timeout for the task to stop gracefully, otherwise abort it
  pub async fn cancel(self) -> Result<(), anyhow::Error> {
    self.f.await
  }
}

/// assuming apub priv key and ids are immutable, then we don't need to have TTL
/// TODO: capacity should be configurable maybe based on memory use
pub(crate) async fn get_actor_cached(
  pool: &mut DbPool<'_>,
  actor_type: ActorType,
  actor_apub_id: &Url,
) -> Result<Arc<SiteOrCommunityOrUser>> {
  static CACHE: Lazy<Cache<Url, Arc<SiteOrCommunityOrUser>>> =
    Lazy::new(|| Cache::builder().max_capacity(10000).build());
  CACHE
    .try_get_with(actor_apub_id.clone(), async {
      let url = actor_apub_id.clone().into();
      let person = match actor_type {
        ActorType::Site => SiteOrCommunityOrUser::Site(
          Site::read_from_apub_id(pool, &url)
            .await?
            .context("apub site not found")?
            .into(),
        ),
        ActorType::Community => SiteOrCommunityOrUser::UserOrCommunity(UserOrCommunity::Community(
          Community::read_from_apub_id(pool, &url)
            .await?
            .context("apub community not found")?
            .into(),
        )),
        ActorType::Person => SiteOrCommunityOrUser::UserOrCommunity(UserOrCommunity::User(
          Person::read_from_apub_id(pool, &url)
            .await?
            .context("apub person not found")?
            .into(),
        )),
      };
      Result::<_, anyhow::Error>::Ok(Arc::new(person))
    })
    .await
    .map_err(|e| anyhow::anyhow!("err getting actor {actor_type:?} {actor_apub_id}: {e:?}"))
}

type CachedActivityInfo = Option<Arc<(SentActivity, SharedInboxActivities)>>;
/// activities are immutable so cache does not need to have TTL
/// May return None if the corresponding id does not exist or is a received activity.
/// Holes in serials are expected behaviour in postgresql
/// todo: cache size should probably be configurable / dependent on desired memory usage
pub(crate) async fn get_activity_cached(
  pool: &mut DbPool<'_>,
  activity_id: ActivityId,
) -> Result<CachedActivityInfo> {
  static ACTIVITIES: Lazy<Cache<ActivityId, CachedActivityInfo>> =
    Lazy::new(|| Cache::builder().max_capacity(10000).build());
  ACTIVITIES
    .try_get_with(activity_id, async {
      let row = SentActivity::read(pool, activity_id)
        .await
        .context("could not read activity")?;
      let Some(mut row) = row else {
        return anyhow::Result::<_, anyhow::Error>::Ok(None);
      };
      // swap to avoid cloning
      let mut data = Value::Null;
      std::mem::swap(&mut row.data, &mut data);
      let activity_actual: SharedInboxActivities = serde_json::from_value(data)?;

      Ok(Some(Arc::new((row, activity_actual))))
    })
    .await
    .map_err(|e| anyhow::anyhow!("err getting activity: {e:?}"))
}

/// return the most current activity id (with 1 second cache)
pub(crate) async fn get_latest_activity_id(pool: &mut DbPool<'_>) -> Result<ActivityId> {
  static CACHE: Lazy<Cache<(), ActivityId>> = Lazy::new(|| {
    Cache::builder()
      .time_to_live(CACHE_DURATION_FEDERATION)
      .build()
  });
  CACHE
    .try_get_with((), async {
      use diesel::dsl::max;
      use lemmy_db_schema::schema::sent_activity::dsl::{id, sent_activity};
      let conn = &mut get_conn(pool).await?;
      let seq: Option<ActivityId> = sent_activity.select(max(id)).get_result(conn).await?;
      let latest_id = seq.unwrap_or(ActivityId(0));
      anyhow::Result::<_, anyhow::Error>::Ok(latest_id as ActivityId)
    })
    .await
    .map_err(|e| anyhow::anyhow!("err getting id: {e:?}"))
}

/// the domain name is needed for logging, pass it to the stats printer so it doesn't need to look
/// up the domain itself
pub(crate) struct FederationQueueStateWithDomain {
  pub domain: String,
  pub state: FederationQueueState,
}
