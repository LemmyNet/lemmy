#![recursion_limit = "512"]
pub mod api_routes;
pub mod code_migrations;
pub mod root_span_builder;
pub mod scheduled_tasks;

#[cfg(feature = "console")]
use console_subscriber::ConsoleLayer;
use lemmy_utils::LemmyError;
use opentelemetry::{
  sdk::{propagation::TraceContextPropagator, Resource},
  KeyValue,
};
use opentelemetry_otlp::WithExportConfig;
use tracing::subscriber::set_global_default;
use tracing_error::ErrorLayer;
use tracing_log::LogTracer;
use tracing_subscriber::{filter::Targets, layer::SubscriberExt, Layer, Registry};

pub fn init_tracing(opentelemetry_url: Option<&str>) -> Result<(), LemmyError> {
  LogTracer::init()?;

  opentelemetry::global::set_text_map_propagator(TraceContextPropagator::new());

  let log_description = std::env::var("RUST_LOG").unwrap_or_else(|_| "info".into());

  let targets = log_description
    .trim()
    .trim_matches('"')
    .parse::<Targets>()?;

  let format_layer = tracing_subscriber::fmt::layer().with_filter(targets.clone());

  #[cfg(feature = "console")]
  let console_layer = ConsoleLayer::builder()
    .with_default_env()
    .server_addr(([0, 0, 0, 0], 6669))
    .event_buffer_capacity(1024 * 1024)
    .spawn();

  let subscriber = Registry::default()
    .with(format_layer)
    .with(ErrorLayer::default());

  #[cfg(feature = "console")]
  let subscriber = subscriber.with(console_layer);

  if let Some(url) = opentelemetry_url {
    let tracer = opentelemetry_otlp::new_pipeline()
      .tracing()
      .with_trace_config(
        opentelemetry::sdk::trace::config()
          .with_resource(Resource::new(vec![KeyValue::new("service.name", "lemmy")])),
      )
      .with_exporter(
        opentelemetry_otlp::new_exporter()
          .tonic()
          .with_endpoint(url),
      )
      .install_batch(opentelemetry::runtime::Tokio)?;

    let otel_layer = tracing_opentelemetry::layer()
      .with_tracer(tracer)
      .with_filter(targets);

    let subscriber = subscriber.with(otel_layer);

    set_global_default(subscriber)?;
  } else {
    set_global_default(subscriber)?;
  }

  Ok(())
}
