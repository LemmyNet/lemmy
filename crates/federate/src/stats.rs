use crate::util::get_latest_activity_id;
use chrono::Local;
use lemmy_api_common::federate_retry_sleep_duration;
use lemmy_db_schema::{
  source::federation_queue_state::FederationQueueState,
  utils::{ActualDbPool, DbPool},
};
use std::{collections::HashMap, time::Duration};
use tokio::{sync::mpsc::UnboundedReceiver, time::interval};
use tracing::{debug, error, info};

/// every 60s, print the state for every instance. exits if the receiver is done (all senders
/// dropped)
pub(crate) async fn receive_print_stats(
  pool: ActualDbPool,
  mut receiver: UnboundedReceiver<(String, FederationQueueState)>,
) {
  let pool = &mut DbPool::Pool(&pool);
  let mut printerval = interval(Duration::from_secs(60));
  let mut stats = HashMap::new();
  loop {
    tokio::select! {
      ele = receiver.recv() => {
        let Some((domain, ele)) = ele else {
          print_stats(pool, &stats).await;
          return;
        };
        stats.insert(domain, ele);
      },
      _ = printerval.tick() => {
        print_stats(pool, &stats).await;
      }
    }
  }
}

async fn print_stats(pool: &mut DbPool<'_>, stats: &HashMap<String, FederationQueueState>) {
  let last_id = get_latest_activity_id(pool).await;
  let Ok(last_id) = last_id else {
    error!("could not get last id");
    return;
  };
  // it's expected that the values are a bit out of date, everything < SAVE_STATE_EVERY should be
  // considered up to date
  info!("Federation state as of {}:", Local::now().to_rfc3339());
  // todo: more stats (act/sec, avg http req duration)
  let mut ok_count = 0;
  let mut behind_count = 0;
  for (domain, stat) in stats {
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
}
