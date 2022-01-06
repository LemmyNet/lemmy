#![recursion_limit = "512"]
pub mod api_routes;
pub mod code_migrations;
pub mod root_span_builder;
pub mod scheduled_tasks;

use lemmy_utils::LemmyError;
use opentelemetry::{
  sdk::{propagation::TraceContextPropagator, Resource},
  KeyValue,
};
use opentelemetry_otlp::WithExportConfig;
use tracing::subscriber::set_global_default;
use tracing_error::ErrorLayer;
use tracing_log::LogTracer;
use tracing_subscriber::{layer::SubscriberExt, EnvFilter, Registry};

pub fn init_tracing(opentelemetry_url: Option<&str>) -> Result<(), LemmyError> {
  LogTracer::init()?;

  opentelemetry::global::set_text_map_propagator(TraceContextPropagator::new());

  let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
  let format_layer = tracing_subscriber::fmt::layer();

  let subscriber = Registry::default()
    .with(env_filter)
    .with(format_layer)
    .with(ErrorLayer::default());

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

    let otel_layer = tracing_opentelemetry::layer().with_tracer(tracer);

    let subscriber = subscriber.with(otel_layer);

    set_global_default(subscriber)?;
  } else {
    set_global_default(subscriber)?;
  }

  Ok(())
}
