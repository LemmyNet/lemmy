#[cfg(feature = "console")]
use console_subscriber::ConsoleLayer;
use lemmy_utils::{error::LemmyResult, VERSION};
use opentelemetry::KeyValue;
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::{propagation::TraceContextPropagator, Resource};
use tracing::{subscriber::set_global_default, Subscriber};
use tracing_subscriber::{filter::Targets, layer::SubscriberExt, registry::LookupSpan, Layer};

pub fn init_tracing<S>(opentelemetry_url: &str, subscriber: S, targets: Targets) -> LemmyResult<()>
where
  S: Subscriber + for<'a> LookupSpan<'a> + Send + Sync + 'static,
{
  opentelemetry::global::set_text_map_propagator(TraceContextPropagator::new());

  #[cfg(feature = "console")]
  let console_layer = ConsoleLayer::builder()
    .with_default_env()
    .server_addr(([0, 0, 0, 0], 6669))
    .event_buffer_capacity(1024 * 1024)
    .spawn();

  #[cfg(feature = "console")]
  let subscriber = subscriber.with(console_layer);

  let tracer = opentelemetry_otlp::new_pipeline()
    .tracing()
    .with_trace_config(
      opentelemetry_sdk::trace::config().with_resource(Resource::new(vec![
        KeyValue::new("service.name", "lemmy"),
        KeyValue::new("service.version", VERSION),
      ])),
    )
    .with_exporter(
      opentelemetry_otlp::new_exporter()
        .tonic()
        .with_endpoint(opentelemetry_url),
    )
    .install_batch(opentelemetry_sdk::runtime::Tokio)?;

  let otel_layer = tracing_opentelemetry::layer()
    .with_tracer(tracer)
    .with_filter(targets);

  let subscriber = subscriber.with(otel_layer);

  set_global_default(subscriber)?;

  Ok(())
}
