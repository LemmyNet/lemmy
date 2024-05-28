use crate::util::get_latest_activity_id;
use chrono::Local;
use diesel::result::Error::NotFound;
use lemmy_api_common::federate_retry_sleep_duration;
use lemmy_db_schema::{
  newtypes::InstanceId,
  source::{federation_queue_state::FederationQueueState, instance::Instance},
  utils::{ActualDbPool, DbPool},
};
use lemmy_utils::{error::LemmyResult, CACHE_DURATION_FEDERATION};
use moka::future::Cache;
use once_cell::sync::Lazy;
use std::{collections::HashMap, time::Duration};
use tokio::{sync::mpsc::UnboundedReceiver, time::interval};
use tracing::{debug, info, warn};

/// every 60s, print the state for every instance. exits if the receiver is done (all senders
/// dropped)
pub(crate) async fn receive_print_stats(
  pool: ActualDbPool,
  mut receiver: UnboundedReceiver<(InstanceId, FederationQueueState)>,
) {
  let pool = &mut DbPool::Pool(&pool);
  let mut printerval = interval(Duration::from_secs(60));
  let mut stats = HashMap::new();
  loop {
    tokio::select! {
      ele = receiver.recv() => {
        match ele {
          // update stats for instance
          Some((instance_id, ele)) => {stats.insert(instance_id, ele);},
          // receiver closed, print stats and exit
          None => {
            print_stats(pool, &stats).await;
            return;
          }
        }
      },
      _ = printerval.tick() => {
        print_stats(pool, &stats).await;
      }
    }
  }
}

async fn print_stats(pool: &mut DbPool<'_>, stats: &HashMap<InstanceId, FederationQueueState>) {
  let res = print_stats_with_error(pool, stats).await;
  if let Err(e) = res {
    warn!("Failed to print stats: {e}");
  }
}

async fn print_stats_with_error(
  pool: &mut DbPool<'_>,
  stats: &HashMap<InstanceId, FederationQueueState>,
) -> LemmyResult<()> {
  static INSTANCE_CACHE: Lazy<Cache<(), Vec<Instance>>> = Lazy::new(|| {
    Cache::builder()
      .max_capacity(1)
      .time_to_live(CACHE_DURATION_FEDERATION)
      .build()
  });
  let instances = INSTANCE_CACHE
    .try_get_with((), async { Instance::read_all(pool).await })
    .await?;

  let last_id = get_latest_activity_id(pool).await?;

  // it's expected that the values are a bit out of date, everything < SAVE_STATE_EVERY should be
  // considered up to date
  info!("Federation state as of {}:", Local::now().to_rfc3339());
  // todo: more stats (act/sec, avg http req duration)
  let mut ok_count = 0;
  let mut behind_count = 0;
  for (instance_id, stat) in stats {
    let domain = &instances
      .iter()
      .find(|i| &i.id == instance_id)
      .ok_or(NotFound)?
      .domain;
    let behind = last_id.0 - stat.last_successful_id.map(|e| e.0).unwrap_or(0);
    if stat.fail_count > 0 {
      info!(
        "{domain}: Warning. {behind} behind, {} consecutive fails, current retry delay {:.2?}",
        stat.fail_count,
        federate_retry_sleep_duration(stat.fail_count)
      );
    } else if behind > 0 {
      debug!("{}: Ok. {} activities behind", domain, behind);
      behind_count += 1;
    } else {
      ok_count += 1;
    }
  }
  info!("{ok_count} others up to date. {behind_count} instances behind.");
  Ok(())
}
