use crate::{
  util::{retry_sleep_duration, CancellableTask},
  worker::InstanceWorker,
};
use activitypub_federation::config::FederationConfig;
use chrono::{Local, Timelike};
use federation_queue_state::FederationQueueState;
use lemmy_api_common::context::LemmyContext;
use lemmy_db_schema::{
  source::instance::Instance,
  utils::{ActualDbPool, DbPool},
};
use std::{collections::HashMap, time::Duration};
use tokio::{
  sync::mpsc::{unbounded_channel, UnboundedReceiver},
  time::sleep,
};
use tokio_util::sync::CancellationToken;

mod federation_queue_state;
mod util;
mod worker;

static WORKER_EXIT_TIMEOUT: Duration = Duration::from_secs(30);
#[cfg(debug_assertions)]
static INSTANCES_RECHECK_DELAY: Duration = Duration::from_secs(5);
#[cfg(not(debug_assertions))]
static INSTANCES_RECHECK_DELAY: Duration = Duration::from_secs(60);

pub struct Opts {
  /// how many processes you are starting in total
  pub process_count: i32,
  /// the index of this process (1-based: 1 - process_count)
  pub process_index: i32,
}

async fn start_stop_federation_workers(
  opts: Opts,
  pool: ActualDbPool,
  federation_config: FederationConfig<LemmyContext>,
  cancel: CancellationToken,
) -> anyhow::Result<()> {
  let mut workers = HashMap::new();

  let (stats_sender, stats_receiver) = unbounded_channel();
  let exit_print = tokio::spawn(receive_print_stats(pool.clone(), stats_receiver));
  let pool2 = &mut DbPool::Pool(&pool);
  let process_index = opts.process_index - 1;
  let local_domain = federation_config.settings().get_hostname_without_port()?;
  loop {
    let mut total_count = 0;
    let mut dead_count = 0;
    let mut disallowed_count = 0;
    for (instance, allowed, is_dead) in Instance::read_all_with_blocked_and_dead(pool2).await? {
      if instance.domain == local_domain {
        continue;
      }
      if instance.id.inner() % opts.process_count != process_index {
        continue;
      }
      total_count += 1;
      if !allowed {
        disallowed_count += 1;
      }
      if is_dead {
        dead_count += 1;
      }
      let should_federate = allowed && !is_dead;
      if !workers.contains_key(&instance.id) && should_federate {
        let stats_sender = stats_sender.clone();
        let context = federation_config.to_request_data();
        let pool = pool.clone();
        workers.insert(
          instance.id,
          CancellableTask::spawn(WORKER_EXIT_TIMEOUT, |stop| async move {
            InstanceWorker::init_and_loop(
              instance,
              context,
              &mut DbPool::Pool(&pool),
              stop,
              stats_sender,
            )
            .await?;
            Ok(())
          }),
        );
      } else if !should_federate {
        if let Some(worker) = workers.remove(&instance.id) {
          if let Err(e) = worker.cancel().await {
            tracing::error!("error stopping worker: {e}");
          }
        }
      }
    }
    let worker_count = workers.len();
    tracing::info!("Federating to {worker_count}/{total_count} instances ({dead_count} dead, {disallowed_count} disallowed)");
    tokio::select! {
      () = sleep(INSTANCES_RECHECK_DELAY) => {},
      _ = cancel.cancelled() => { break; }
    }
  }
  drop(stats_sender);
  tracing::warn!(
    "Waiting for {} workers ({:.2?} max)",
    workers.len(),
    WORKER_EXIT_TIMEOUT
  );
  // the cancel futures need to be awaited concurrently for the shutdown processes to be triggered concurrently
  futures::future::join_all(workers.into_values().map(util::CancellableTask::cancel)).await;
  exit_print.await?;
  Ok(())
}

/// starts and stops federation workers depending on which instances are on db
/// await the returned future to stop/cancel all workers gracefully
pub fn start_stop_federation_workers_cancellable(
  opts: Opts,
  pool: ActualDbPool,
  config: FederationConfig<LemmyContext>,
) -> CancellableTask<()> {
  CancellableTask::spawn(WORKER_EXIT_TIMEOUT, move |c| {
    start_stop_federation_workers(opts, pool, config, c)
  })
}

/// every 60s, print the state for every instance. exits if the receiver is done (all senders dropped)
async fn receive_print_stats(
  pool: ActualDbPool,
  mut receiver: UnboundedReceiver<FederationQueueState>,
) {
  let pool = &mut DbPool::Pool(&pool);
  let mut printerval = tokio::time::interval(Duration::from_secs(60));
  printerval.tick().await; // skip first
  let mut stats = HashMap::new();
  loop {
    tokio::select! {
      ele = receiver.recv() => {
        let Some(ele) = ele else {
          tracing::info!("done. quitting");
          print_stats(pool, &stats).await;
          return;
        };
        stats.insert(ele.domain.clone(), ele);
      },
      _ = printerval.tick() => {
        print_stats(pool, &stats).await;
      }
    }
  }
}

async fn print_stats(pool: &mut DbPool<'_>, stats: &HashMap<String, FederationQueueState>) {
  let last_id = crate::util::get_latest_activity_id(pool).await;
  let Ok(last_id) = last_id else {
    tracing::error!("could not get last id");
    return;
  };
  // it's expected that the values are a bit out of date, everything < SAVE_STATE_EVERY should be considered up to date
  tracing::info!(
    "Federation state as of {}:",
    Local::now()
      .with_nanosecond(0)
      .expect("0 is valid nanos")
      .to_rfc3339()
  );
  // todo: less noisy output (only output failing instances and summary for successful)
  // todo: more stats (act/sec, avg http req duration)
  let mut ok_count = 0;
  for stat in stats.values() {
    let behind = last_id - stat.last_successful_id;
    if stat.fail_count > 0 {
      tracing::info!(
        "{}: Warning. {} behind, {} consecutive fails, current retry delay {:.2?}",
        stat.domain,
        behind,
        stat.fail_count,
        retry_sleep_duration(stat.fail_count)
      );
    } else if behind > 0 {
      tracing::info!("{}: Ok. {} behind", stat.domain, behind);
    } else {
      ok_count += 1;
    }
  }
  tracing::info!("{ok_count} others up to date");
}
