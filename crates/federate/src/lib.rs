use crate::{util::CancellableTask, worker::InstanceWorker};
use activitypub_federation::config::FederationConfig;
use lemmy_api_common::context::LemmyContext;
use lemmy_db_schema::{
  newtypes::InstanceId,
  source::{federation_queue_state::FederationQueueState, instance::Instance},
};
use lemmy_utils::error::LemmyResult;
use stats::receive_print_stats;
use std::{collections::HashMap, time::Duration};
use tokio::{
  sync::mpsc::{unbounded_channel, UnboundedSender},
  task::JoinHandle,
  time::sleep,
};
use tokio_util::sync::CancellationToken;
use tracing::info;

mod stats;
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
    assert!(opts.process_count > 0);
    assert!(opts.process_index > 0);
    assert!(opts.process_index <= opts.process_count);

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
    CancellableTask::spawn(WORKER_EXIT_TIMEOUT, move |cancel| async move {
      self.do_loop(cancel).await.unwrap();
      self.cancel().await.unwrap();
    })
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

#[cfg(test)]
#[allow(clippy::unwrap_used)]
#[allow(clippy::indexing_slicing)]
mod test {

  use super::*;
  use activitypub_federation::config::Data;
  use chrono::DateTime;
  use lemmy_db_schema::source::{
    federation_allowlist::FederationAllowList,
    federation_blocklist::FederationBlockList,
    instance::InstanceForm,
  };
  use lemmy_utils::error::LemmyError;
  use serial_test::serial;
  use std::{
    collections::HashSet,
    sync::{Arc, Mutex},
  };
  use tokio::{spawn, time::sleep};

  struct TestData {
    send_manager: SendManager,
    context: Data<LemmyContext>,
    instances: Vec<Instance>,
  }
  impl TestData {
    async fn init(process_count: i32, process_index: i32) -> LemmyResult<Self> {
      let context = LemmyContext::init_test_context().await;
      let opts = Opts {
        process_count,
        process_index,
      };
      let federation_config = FederationConfig::builder()
        .domain("local.com")
        .app_data(context.clone())
        .build()
        .await?;

      let pool = &mut context.pool();
      let instances = vec![
        Instance::read_or_create(pool, "alpha.com".to_string()).await?,
        Instance::read_or_create(pool, "beta.com".to_string()).await?,
        Instance::read_or_create(pool, "gamma.com".to_string()).await?,
      ];

      let send_manager = SendManager::new(opts, federation_config);
      Ok(Self {
        send_manager,
        context,
        instances,
      })
    }

    async fn run(&mut self) -> LemmyResult<()> {
      // start it and cancel after workers are running
      let cancel = CancellationToken::new();
      let cancel_ = cancel.clone();
      spawn(async move {
        sleep(Duration::from_millis(100)).await;
        cancel_.cancel();
      });
      self.send_manager.do_loop(cancel.clone()).await?;
      Ok(())
    }

    async fn cleanup(self) -> LemmyResult<()> {
      self.send_manager.cancel().await?;
      Instance::delete_all(&mut self.context.pool()).await?;
      Ok(())
    }
  }

  /// Basic test with default params and only active/allowed instances
  #[tokio::test]
  #[serial]
  async fn test_send_manager() -> LemmyResult<()> {
    let mut data = TestData::init(1, 1).await?;

    data.run().await?;
    assert_eq!(3, data.send_manager.workers.len());
    let workers: HashSet<_> = data.send_manager.workers.keys().cloned().collect();
    let instances: HashSet<_> = data.instances.iter().map(|i| i.id).collect();
    assert_eq!(instances, workers);

    data.cleanup().await?;
    Ok(())
  }

  /// Running with multiple processes should start correct workers
  #[tokio::test]
  #[serial]
  async fn test_send_manager_processes() -> LemmyResult<()> {
    let active = Arc::new(Mutex::new(vec![]));
    let execute = |count, index, active: Arc<Mutex<Vec<InstanceId>>>| async move {
      let mut data = TestData::init(count, index).await?;
      data.run().await?;
      assert_eq!(1, data.send_manager.workers.len());
      for k in data.send_manager.workers.keys() {
        active.lock().unwrap().push(*k);
      }
      data.cleanup().await?;
      Ok::<(), LemmyError>(())
    };
    execute(3, 1, active.clone()).await?;
    execute(3, 2, active.clone()).await?;
    execute(3, 3, active.clone()).await?;

    // Should run exactly three workers
    assert_eq!(3, active.lock().unwrap().len());

    Ok(())
  }

  /// Use blocklist, should not send to blocked instances
  #[tokio::test]
  #[serial]
  async fn test_send_manager_blocked() -> LemmyResult<()> {
    let mut data = TestData::init(1, 1).await?;

    let domain = data.instances[0].domain.clone();
    FederationBlockList::replace(&mut data.context.pool(), Some(vec![domain])).await?;
    data.run().await?;
    let workers = &data.send_manager.workers;
    assert_eq!(2, workers.len());
    assert!(workers.contains_key(&data.instances[1].id));
    assert!(workers.contains_key(&data.instances[2].id));

    data.cleanup().await?;
    Ok(())
  }

  /// Use allowlist, should only send to allowed instance
  #[tokio::test]
  #[serial]
  async fn test_send_manager_allowed() -> LemmyResult<()> {
    let mut data = TestData::init(1, 1).await?;

    let domain = data.instances[0].domain.clone();
    FederationAllowList::replace(&mut data.context.pool(), Some(vec![domain])).await?;
    data.run().await?;
    let workers = &data.send_manager.workers;
    assert_eq!(1, workers.len());
    assert!(workers.contains_key(&data.instances[0].id));

    data.cleanup().await?;
    Ok(())
  }

  /// Mark instance as dead, there should be no worker created for it
  #[tokio::test]
  #[serial]
  async fn test_send_manager_dead() -> LemmyResult<()> {
    let mut data = TestData::init(1, 1).await?;

    let instance = &data.instances[0];
    let form = InstanceForm::builder()
      .domain(instance.domain.clone())
      .updated(DateTime::from_timestamp(0, 0))
      .build();
    Instance::update(&mut data.context.pool(), instance.id, form).await?;

    data.run().await?;
    let workers = &data.send_manager.workers;
    assert_eq!(2, workers.len());
    assert!(workers.contains_key(&data.instances[1].id));
    assert!(workers.contains_key(&data.instances[2].id));

    data.cleanup().await?;
    Ok(())
  }
}
