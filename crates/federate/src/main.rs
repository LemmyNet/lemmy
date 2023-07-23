use crate::{
  util::{retry_sleep_duration, spawn_cancellable},
  worker::instance_worker,
};
use activitypub_federation::config::FederationConfig;
use chrono::{Local, Timelike};
use federation_queue_state::FederationQueueState;
use lemmy_api_common::request::build_user_agent;
use lemmy_apub::{VerifyUrlData, FEDERATION_HTTP_FETCH_LIMIT};
use lemmy_db_schema::{
  source::instance::Instance,
  utils::{build_db_pool, DbPool},
};
use lemmy_utils::{error::LemmyErrorExt2, settings::SETTINGS, REQWEST_TIMEOUT};
use reqwest::Client;
use reqwest_middleware::ClientBuilder;
use reqwest_tracing::TracingMiddleware;
use std::{collections::HashMap, time::Duration};
use tokio::{
  signal::unix::SignalKind,
  sync::mpsc::{unbounded_channel, UnboundedReceiver},
  time::sleep,
};

mod federation_queue_state;
mod util;
mod worker;

static WORKER_EXIT_TIMEOUT: Duration = Duration::from_secs(30);

#[tokio::main]
async fn main() -> anyhow::Result<()> {
  tracing_subscriber::fmt::init();
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
  let process_num = 1 - 1; // todo: pass these in via command line args
  let process_count = 1;
  let mut workers = HashMap::new();
  let mut pool2 = DbPool::from(&pool);

  let (stats_sender, stats_receiver) = unbounded_channel();
  let exit_print = tokio::spawn(receive_print_stats(&mut pool2, stats_receiver));
  let mut interrupt = tokio::signal::unix::signal(SignalKind::interrupt())?;
  let mut terminate = tokio::signal::unix::signal(SignalKind::terminate())?;
  loop {
    for (instance, should_federate) in Instance::read_all_with_blocked(&mut pool2)
      .await?
      .into_iter()
    {
      if instance.id.inner() % process_count != process_num {
        continue;
      }
      if !workers.contains_key(&instance.id) && should_federate {
        let stats_sender = stats_sender.clone();
        workers.insert(
          instance.id,
          spawn_cancellable(WORKER_EXIT_TIMEOUT, |stop| {
            instance_worker(
              pool2,
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
      _ = tokio::signal::ctrl_c() => {
        tracing::warn!("Received ctrl-c, shutting down gracefully...");
        break;
      }
      _ = interrupt.recv() => {
        tracing::warn!("Received interrupt, shutting down gracefully...");
        break;
      }
      _ = terminate.recv() => {
        tracing::warn!("Received terminate, shutting down gracefully...");
        break;
      }
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

/// every 60s, print the state for every instance. exits if the receiver is done (all senders dropped)
async fn receive_print_stats(
  mut pool: &mut DbPool<'_>,
  mut receiver: UnboundedReceiver<FederationQueueState>,
) {
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
