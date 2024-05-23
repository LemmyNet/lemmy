use crate::{
  util::{get_latest_activity_id, CancellableTask},
  worker::InstanceWorker,
};
use activitypub_federation::config::FederationConfig;
use chrono::{Local, Timelike};
use lemmy_api_common::{context::LemmyContext, federate_retry_sleep_duration};
use lemmy_db_schema::{
  newtypes::InstanceId,
  source::{federation_queue_state::FederationQueueState, instance::Instance},
  utils::{ActualDbPool, DbPool},
};
use lemmy_utils::error::LemmyResult;
use std::{collections::HashMap, time::Duration};
use tokio::{
  sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender},
  task::JoinHandle,
  time::{interval, sleep},
};
use tokio_util::sync::CancellationToken;
use tracing::{debug, error, info};

mod util;
mod worker;

static WORKER_EXIT_TIMEOUT: Duration = Duration::from_secs(30);
#[cfg(debug_assertions)]
static INSTANCES_RECHECK_DELAY: Duration = Duration::from_secs(5);
#[cfg(not(debug_assertions))]
static INSTANCES_RECHECK_DELAY: Duration = Duration::from_secs(60);

#[derive(Clone)]
pub struct Opts {
  /// how many processes you are starting in total
  pub process_count: i32,
  /// the index of this process (1-based: 1 - process_count)
  pub process_index: i32,
}

pub struct SendManager {
  opts: Opts,
  workers: HashMap<InstanceId, CancellableTask>,
  context: FederationConfig<LemmyContext>,
  stats_sender: UnboundedSender<(String, FederationQueueState)>,
  exit_print: JoinHandle<()>,
}

impl SendManager {
  pub fn new(opts: Opts, context: FederationConfig<LemmyContext>) -> Self {
    let (stats_sender, stats_receiver) = unbounded_channel();
    Self {
      opts,
      workers: HashMap::new(),
      stats_sender,
      exit_print: tokio::spawn(receive_print_stats(
        context.inner_pool().clone(),
        stats_receiver,
      )),
      context,
    }
  }

  pub fn run(mut self) -> CancellableTask {
    let task = CancellableTask::spawn(WORKER_EXIT_TIMEOUT, move |cancel| async move {
      self.do_loop(cancel).await.unwrap();
      self.cancel().await.unwrap();
    });
    task
  }

  async fn do_loop(&mut self, cancel: CancellationToken) -> LemmyResult<()> {
    let process_index = self.opts.process_index - 1;
    info!(
      "Starting federation workers for process count {} and index {}",
      self.opts.process_count, process_index
    );
    let local_domain = self.context.settings().get_hostname_without_port()?;
    let mut pool = self.context.pool();
    loop {
      let mut total_count = 0;
      let mut dead_count = 0;
      let mut disallowed_count = 0;
      for (instance, allowed, is_dead) in
        Instance::read_federated_with_blocked_and_dead(&mut pool).await?
      {
        if instance.domain == local_domain {
          continue;
        }
        if instance.id.inner() % self.opts.process_count != process_index {
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
        if should_federate {
          if self.workers.contains_key(&instance.id) {
            // worker already running
            continue;
          }
          // create new worker
          let instance = instance.clone();
          let req_data = self.context.to_request_data();
          let stats_sender = self.stats_sender.clone();
          self.workers.insert(
            instance.id,
            CancellableTask::spawn(WORKER_EXIT_TIMEOUT, move |stop| async move {
              InstanceWorker::init_and_loop(instance, req_data, stop, stats_sender).await
            }),
          );
        } else if !should_federate {
          if let Some(worker) = self.workers.remove(&instance.id) {
            if let Err(e) = worker.cancel().await {
              tracing::error!("error stopping worker: {e}");
            }
          }
        }
      }
      let worker_count = self.workers.len();
      tracing::info!("Federating to {worker_count}/{total_count} instances ({dead_count} dead, {disallowed_count} disallowed)");
      tokio::select! {
        () = sleep(INSTANCES_RECHECK_DELAY) => {},
        _ = cancel.cancelled() => { return Ok(()) }
      }
    }
  }

  pub async fn cancel(self) -> LemmyResult<()> {
    drop(self.stats_sender);
    tracing::warn!(
      "Waiting for {} workers ({:.2?} max)",
      self.workers.len(),
      WORKER_EXIT_TIMEOUT
    );
    // the cancel futures need to be awaited concurrently for the shutdown processes to be triggered concurrently
    futures::future::join_all(
      self
        .workers
        .into_values()
        .map(util::CancellableTask::cancel),
    )
    .await;
    self.exit_print.await?;
    Ok(())
  }
}

/// every 60s, print the state for every instance. exits if the receiver is done (all senders dropped)
async fn receive_print_stats(
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
  // it's expected that the values are a bit out of date, everything < SAVE_STATE_EVERY should be considered up to date
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

#[cfg(test)]
#[allow(clippy::unwrap_used)]
#[allow(clippy::indexing_slicing)]
mod test {

  use super::*;

  #[tokio::test]
  async fn test_start_stop_federation_workers() -> LemmyResult<()> {
    // initialization
    let context = LemmyContext::init_test_context().await;
    let pool = &mut context.pool();
    let opts = Opts {
      process_count: 1,
      process_index: 1,
    };
    let federation_config = FederationConfig::builder()
      .domain("local.com")
      .app_data(context.clone())
      .build()
      .await?;

    let instances = vec![
      Instance::read_or_create(pool, "alpha.com".to_string()).await?,
      Instance::read_or_create(pool, "beta.com".to_string()).await?,
      Instance::read_or_create(pool, "gamma.com".to_string()).await?,
    ];

    // start it and wait a moment
    let task = SendManager::new(opts, federation_config);
    task.run();
    sleep(Duration::from_secs(1));

    // check that correct number of instance workers was started
    // TODO: need to wrap in Arc or something similar
    // TODO: test with different `opts`, dead/blocked instances etc
    assert_eq!(3, task.workers.len());

    // cleanup
    for i in instances {
      Instance::delete(pool, i.id).await?;
    }
    task.cancel().await?;
    Ok(())
  }
}
