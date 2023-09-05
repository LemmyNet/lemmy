use console_subscriber::ConsoleLayer;
use lemmy_utils::error::LemmyError;
use opentelemetry::{
    sdk::{propagation::TraceContextPropagator, Resource},
    KeyValue,
};
use opentelemetry_otlp::WithExportConfig;
use tracing::{subscriber::set_global_default, Subscriber};
use tracing_subscriber::{filter::Targets, layer::SubscriberExt, registry::LookupSpan, Layer};

pub fn init_tracing<S>(
    opentelemetry_url: &str,
    subscriber: S,
    targets: Targets,
) -> Result<(), LemmyError>
where
    S: Subscriber + for<'a> LookupSpan<'a> + Send + Sync + 'static,
{
    opentelemetry::global::set_text_map_propagator(TraceContextPropagator::new());

    let console_layer = ConsoleLayer::builder()
        .with_default_env()
        .server_addr(([0, 0, 0, 0], 6669))
        .event_buffer_capacity(1024 * 1024)
        .spawn();

    let subscriber = subscriber.with(console_layer);

    let tracer = opentelemetry_otlp::new_pipeline()
        .tracing()
        .with_trace_config(
            opentelemetry::sdk::trace::config()
                .with_resource(Resource::new(vec![KeyValue::new("service.name", "lemmy")])),
        )
        .with_exporter(
            opentelemetry_otlp::new_exporter()
                .tonic()
                .with_endpoint(opentelemetry_url),
        )
        .install_batch(opentelemetry::runtime::Tokio)?;

    let otel_layer = tracing_opentelemetry::layer()
        .with_tracer(tracer)
        .with_filter(targets);

    let subscriber = subscriber.with(otel_layer);

    set_global_default(subscriber)?;

    Ok(())
}
