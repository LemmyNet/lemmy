use anyhow::{anyhow, Context, Result};
use dashmap::DashSet;
use diesel::{prelude::*, sql_types::Int8};
use diesel_async::RunQueryDsl;
use lemmy_apub::{
  activity_lists::SharedInboxActivities,
  fetcher::{site_or_community_or_user::SiteOrCommunityOrUser, user_or_community::UserOrCommunity},
};
use lemmy_db_schema::{
  source::{
    activity::{ActorType, SentActivity},
    community::Community,
    person::Person,
    site::Site,
  },
  traits::{ApubActor, Crud},
  utils::{get_conn, DbPool},
};
use moka::future::Cache;
use once_cell::sync::Lazy;
use reqwest::Url;
use serde_json::Value;
use std::{
  borrow::{Borrow, Cow},
  future::Future,
  sync::Arc,
  time::Duration,
};
use tokio::{task::JoinHandle, time::sleep};
use tokio_util::sync::CancellationToken;

/// spawn a task but with graceful shutdown
///
/// only await the returned future when you want to cancel the task
pub fn spawn_cancellable<R: Send + 'static, F>(
  timeout: Duration,
  task: impl FnOnce(CancellationToken) -> F,
) -> impl Future<Output = Result<R>>
where
  F: Future<Output = Result<R>> + Send + 'static,
{
  let stop = CancellationToken::new();
  let task = task(stop.clone());
  let task: JoinHandle<Result<R>> = tokio::spawn(async move {
    match task.await {
      Ok(o) => Ok(o),
      Err(e) => {
        tracing::error!("worker errored out: {e}");
        // todo: if this error happens, requeue worker creation in main
        Err(e)
      }
    }
  });
  let abort = task.abort_handle();
  async move {
    tracing::info!("Shutting down task");
    stop.cancel();
    tokio::select! {
        r = task => {
            Ok(r.context("could not join")??)
        },
        _ = sleep(timeout) => {
            abort.abort();
            tracing::warn!("Graceful shutdown timed out, aborting task");
            Err(anyhow!("task aborted due to timeout"))
        }
    }
  }
}

/// assuming apub priv key and ids are immutable, then we don't need to have TTL
/// TODO: capacity should be configurable maybe based on memory use
pub async fn get_actor_cached(
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
    .map_err(|e| anyhow::anyhow!("err getting actor: {e}"))
}

/// intern urls to reduce memory usage
/// not sure if worth it
pub fn intern_url<'a>(url: impl Into<Cow<'a, Url>>) -> Arc<Url> {
  let url: Cow<'a, Url> = url.into();
  static INTERNED_URLS: Lazy<DashSet<Arc<Url>>> = Lazy::new(DashSet::new);
  return INTERNED_URLS
    .get::<Url>(url.borrow())
    .map(|e| e.clone())
    .unwrap_or_else(|| {
      let ret = Arc::new(url.into_owned());
      INTERNED_URLS.insert(ret.clone());
      ret
    });
}

/// this should maybe be a newtype like all the other PersonId CommunityId etc.
pub type ActivityId = i64;

/// activities are immutable so cache does not need to have TTL
/// May return None if the corresponding id does not exist or is a received activity.
/// Holes in serials are expected behaviour in postgresql
/// todo: cache size should probably be configurable / dependent on desired memory usage
pub async fn get_activity_cached(
  pool: &mut DbPool<'_>,
  activity_id: ActivityId,
) -> Result<Option<Arc<(SentActivity, SharedInboxActivities)>>> {
  static ACTIVITIES: Lazy<Cache<ActivityId, Option<Arc<(SentActivity, SharedInboxActivities)>>>> =
    Lazy::new(|| Cache::builder().max_capacity(10000).build());
  ACTIVITIES
    .try_get_with(activity_id, async {
      let row = SentActivity::read(pool, activity_id)
        .await
        .optional()
        .context("could not read activity")?;
      let Some(mut row) = row else { return anyhow::Result::<_, anyhow::Error>::Ok(None) };
      // swap to avoid cloning
      let mut data = Value::Null;
      std::mem::swap(&mut row.data, &mut data);
      let activity_actual: SharedInboxActivities = serde_json::from_value(data)?;

      Ok(Some(Arc::new((row, activity_actual))))
    })
    .await
    .map_err(|e| anyhow::anyhow!("err getting activity: {e}"))
}

/// return the most current activity id (with 1 second cache)
pub async fn get_latest_activity_id(pool: &mut DbPool<'_>) -> Result<ActivityId> {
  static CACHE: Lazy<Cache<(), ActivityId>> = Lazy::new(|| {
    Cache::builder()
      .time_to_live(Duration::from_secs(1))
      .build()
  });
  CACHE
    .try_get_with((), async {
      let conn = &mut get_conn(pool).await?;
      let Sequence {
        last_value: latest_id,
      } = diesel::sql_query("select last_value from sent_activity_id_seq")
        .get_result(conn)
        .await?;
      anyhow::Result::<_, anyhow::Error>::Ok(latest_id as ActivityId)
    })
    .await
    .map_err(|e| anyhow::anyhow!("err getting id: {e}"))
}

/// how long to sleep based on how many retries have already happened
pub fn retry_sleep_duration(retry_count: i32) -> Duration {
  Duration::from_secs_f64(10.0 * 2.0_f64.powf(retry_count as f64))
}

#[derive(QueryableByName)]
struct Sequence {
  #[diesel(sql_type = Int8)]
  last_value: i64, // this value is bigint for some reason even if sequence is int4
}
