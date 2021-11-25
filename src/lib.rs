#![recursion_limit = "512"]
pub mod api_routes;
pub mod code_migrations;
pub mod scheduled_tasks;

use lemmy_utils::LemmyError;
use tracing::subscriber::set_global_default;
use tracing_error::ErrorLayer;
use tracing_log::LogTracer;
use tracing_subscriber::{layer::SubscriberExt, EnvFilter, Registry};

pub fn init_tracing() -> Result<(), LemmyError> {
  LogTracer::init()?;

  let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
  let format_layer = tracing_subscriber::fmt::layer().pretty();

  let subscriber = Registry::default()
    .with(env_filter)
    .with(format_layer)
    .with(ErrorLayer::default());

  set_global_default(subscriber)?;

  Ok(())
}
