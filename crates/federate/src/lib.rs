use crate::{
  util::{retry_sleep_duration, CancellableTask},
  worker::instance_worker,
};
use activitypub_federation::config::FederationConfig;
use chrono::{Local, Timelike};
use clap::Parser;
use federation_queue_state::FederationQueueState;
use lemmy_db_schema::{
  source::instance::Instance,
  utils::{ActualDbPool, DbPool},
};
use std::{
  collections::{HashMap, HashSet},
  time::Duration,
};
use tokio::{
  sync::mpsc::{unbounded_channel, UnboundedReceiver},
  time::sleep,
};
use tokio_util::sync::CancellationToken;

mod federation_queue_state;
mod util;
mod worker;

static WORKER_EXIT_TIMEOUT: Duration = Duration::from_secs(30);

#[derive(Parser, Debug)]
pub struct Opts {
  /// how many processes you are starting in total
  #[arg(default_value_t = 1)]
  pub process_count: i32,
  /// the index of this process (1-based: 1 - process_count)
  #[arg(default_value_t = 1)]
  pub process_index: i32,
}

async fn start_stop_federation_workers<T: Clone + Send + Sync + 'static>(
  opts: Opts,
  pool: ActualDbPool,
  federation_config: FederationConfig<T>,
  cancel: CancellationToken,
) -> anyhow::Result<()> {
  let mut workers = HashMap::new();

  let (stats_sender, stats_receiver) = unbounded_channel();
  let exit_print = tokio::spawn(receive_print_stats(pool.clone(), stats_receiver));
  let pool2 = &mut DbPool::Pool(&pool);
  let process_index = opts.process_index - 1;
  loop {
    let dead: HashSet<String> = HashSet::from_iter(Instance::dead_instances(pool2).await?);
    let mut total_count = 0;
    let mut dead_count = 0;
    let mut disallowed_count = 0;
    for (instance, allowed) in Instance::read_all_with_blocked(pool2).await?.into_iter() {
      if instance.id.inner() % opts.process_count != process_index {
        continue;
      }
      total_count += 1;
      if !allowed {
        disallowed_count += 1;
      }
      let is_dead = dead.contains(&instance.domain);
      if is_dead {
        dead_count += 1;
      }
      let should_federate = allowed && !is_dead;
      if !workers.contains_key(&instance.id) && should_federate {
        let stats_sender = stats_sender.clone();
        workers.insert(
          instance.id,
          CancellableTask::spawn(WORKER_EXIT_TIMEOUT, |stop| {
            instance_worker(
              pool.clone(),
              instance,
              federation_config.to_request_data(),
              stop,
              stats_sender,
            )
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
      () = sleep(Duration::from_secs(60)) => {},
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
  futures::future::join_all(workers.into_values().map(|e| e.cancel())).await;
  exit_print.await?;
  Ok(())
}

/// starts and stops federation workers depending on which instances are on db
/// await the returned future to stop/cancel all workers gracefully
pub fn start_stop_federation_workers_cancellable(
  opts: Opts,
  pool: ActualDbPool,
  config: FederationConfig<impl Clone + Send + Sync + 'static>,
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
  let mut pool = &mut DbPool::Pool(&pool);
  let mut printerval = tokio::time::interval(Duration::from_secs(60));
  printerval.tick().await; // skip first
  let mut stats = HashMap::new();
  loop {
    tokio::select! {
      ele = receiver.recv() => {
        let Some(ele) = ele else {
          tracing::info!("done. quitting");
          print_stats(&mut pool, &stats).await;
          return;
        };
        stats.insert(ele.domain.clone(), ele);
      },
      _ = printerval.tick() => {
        print_stats(&mut pool, &stats).await;
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
    Local::now().with_nanosecond(0).unwrap().to_rfc3339()
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
    } else {
      if behind > 0 {
        tracing::info!("{}: Ok. {} behind", stat.domain, behind);
      } else {
        ok_count += 1;
      }
    }
  }
  tracing::info!("{ok_count} others up to date");
}
