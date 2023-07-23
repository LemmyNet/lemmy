use crate::{
  util::{retry_sleep_duration, spawn_cancellable},
  worker::instance_worker,
};
use activitypub_federation::config::FederationConfig;
use chrono::{Local, Timelike};
use clap::Parser;
use federation_queue_state::FederationQueueState;
use futures::Future;
use lemmy_api_common::request::build_user_agent;
use lemmy_apub::{VerifyUrlData, FEDERATION_HTTP_FETCH_LIMIT};
use lemmy_db_schema::{
  source::instance::Instance,
  utils::{build_db_pool, ActualDbPool, DbPool},
};
use lemmy_utils::{error::LemmyErrorExt2, settings::SETTINGS, REQWEST_TIMEOUT};
use reqwest::Client;
use reqwest_middleware::ClientBuilder;
use reqwest_tracing::TracingMiddleware;
use std::{
  collections::{HashMap, HashSet},
  time::Duration,
};
use tokio::{
  signal::unix::SignalKind,
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

/// starts and stops federation workers depending on which instances are on db
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
  loop {
    let dead: HashSet<String> = HashSet::from_iter(Instance::dead_instances(pool2).await?);
    for (instance, allowed) in Instance::read_all_with_blocked(pool2).await?.into_iter() {
      if instance.id.inner() % opts.process_count != opts.process_index {
        continue;
      }
      let should_federate = allowed && !dead.contains(&instance.domain);
      if !workers.contains_key(&instance.id) && should_federate {
        let stats_sender = stats_sender.clone();
        workers.insert(
          instance.id,
          spawn_cancellable(WORKER_EXIT_TIMEOUT, |stop| {
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
          if let Err(e) = worker.await {
            tracing::error!("error stopping worker: {e}");
          }
        }
      }
    }
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
  futures::future::join_all(workers.into_values()).await;
  exit_print.await?;
  Ok(())
}

pub fn start_stop_federation_workers_cancellable(
  opts: Opts,
  pool: ActualDbPool,
  config: FederationConfig<impl Clone + Send + Sync + 'static>,
) -> impl Future<Output = anyhow::Result<()>> {
  spawn_cancellable(WORKER_EXIT_TIMEOUT, move |c| {
    start_stop_federation_workers(opts, pool, config, c)
  })
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
  tracing_subscriber::fmt::init();
  let opts = Opts::parse();
  let settings = SETTINGS.to_owned();
  // TODO: wait until migrations are applied? or are they safe from race conditions and i can just call run_migrations here as well?
  let pool = build_db_pool(&settings).await.into_anyhow()?;
  let user_agent = build_user_agent(&settings);
  let reqwest_client = Client::builder()
    .user_agent(user_agent.clone())
    .timeout(REQWEST_TIMEOUT)
    .connect_timeout(REQWEST_TIMEOUT)
    .build()?;

  let client = ClientBuilder::new(reqwest_client.clone())
    .with(TracingMiddleware::default())
    .build();

  let federation_config = FederationConfig::builder()
    .domain(settings.hostname.clone())
    .app_data(())
    .client(client.clone())
    .http_fetch_limit(FEDERATION_HTTP_FETCH_LIMIT)
    .http_signature_compat(true)
    .url_verifier(Box::new(VerifyUrlData(pool.clone())))
    .build()
    .await?;
  let mut interrupt = tokio::signal::unix::signal(SignalKind::interrupt())?;
  let mut terminate = tokio::signal::unix::signal(SignalKind::terminate())?;

  let cancel = start_stop_federation_workers_cancellable(opts, pool, federation_config);
  tokio::select! {
    _ = tokio::signal::ctrl_c() => {
      tracing::warn!("Received ctrl-c, shutting down gracefully...");
    }
    _ = interrupt.recv() => {
      tracing::warn!("Received interrupt, shutting down gracefully...");
    }
    _ = terminate.recv() => {
      tracing::warn!("Received terminate, shutting down gracefully...");
    }
  }
  cancel.await?;
  Ok(())
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
      tracing::info!("{}: Ok. {} behind", stat.domain, behind);
    }
  }
}
