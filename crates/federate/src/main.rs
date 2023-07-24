use activitypub_federation::config::FederationConfig;
use clap::Parser;
use lemmy_api_common::request::build_user_agent;
use lemmy_apub::{VerifyUrlData, FEDERATION_HTTP_FETCH_LIMIT};
use lemmy_db_schema::utils::build_db_pool;
use lemmy_federate::Opts;
use lemmy_utils::{error::LemmyErrorExt2, settings::SETTINGS, REQWEST_TIMEOUT};
use reqwest::Client;
use reqwest_middleware::ClientBuilder;
use reqwest_tracing::TracingMiddleware;
use tokio::signal::unix::SignalKind;

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

  let cancel =
    lemmy_federate::start_stop_federation_workers_cancellable(opts, pool, federation_config);
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
