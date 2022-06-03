#![recursion_limit = "512"]
pub mod api_routes;
pub mod code_migrations;
pub mod root_span_builder;
pub mod scheduled_tasks;
#[cfg(feature = "console")]
pub mod telemetry;

use lemmy_utils::error::LemmyError;
use tracing::subscriber::set_global_default;
use tracing_error::ErrorLayer;
use tracing_log::LogTracer;
use tracing_subscriber::{filter::Targets, layer::SubscriberExt, Layer, Registry};

pub fn init_logging(opentelemetry_url: Option<&str>) -> Result<(), LemmyError> {
  LogTracer::init()?;

  let log_description = std::env::var("RUST_LOG").unwrap_or_else(|_| "info".into());

  let targets = log_description
    .trim()
    .trim_matches('"')
    .parse::<Targets>()?;

  let format_layer = tracing_subscriber::fmt::layer().with_filter(targets.clone());

  let subscriber = Registry::default()
    .with(format_layer)
    .with(ErrorLayer::default());

  if let Some(_url) = opentelemetry_url {
    #[cfg(feature = "console")]
    crate::telemetry::init_tracing(_url, subscriber, targets)?;
    #[cfg(not(feature = "console"))]
    tracing::error!("Feature `console` must be enabled for opentelemetry tracing");
  } else {
    set_global_default(subscriber)?;
  }

  Ok(())
}
