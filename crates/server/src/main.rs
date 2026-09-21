use clap::Parser;
use lemmy_server::{CmdArgs, start_lemmy_server};
use lemmy_utils::{error::LemmyResult, settings::SETTINGS};
use opentelemetry::{
  global::{self, BoxedTracer},
  trace::{Span, SpanKind, Status, Tracer, TracerProvider},
};
use opentelemetry_otlp::{Protocol, WithExportConfig};
use opentelemetry_sdk::trace::SdkTracerProvider;
// use opentelemetry_stdout::SpanExporter;
use std::sync::OnceLock;
use tracing::{level_filters::LevelFilter, subscriber::set_default};
use tracing_subscriber::{EnvFilter, Registry, fmt, layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
pub async fn main() -> LemmyResult<()> {
  let otlp_exporter = opentelemetry_otlp::SpanExporter::builder()
    .with_tonic()
    .build()?;

  // Create a tracer provider with the exporter
  let provider = opentelemetry_sdk::trace::SdkTracerProvider::builder()
    .with_batch_exporter(otlp_exporter)
    .build();

  let tracer = provider.tracer("Lemmy Tracer");

  let telemetry = tracing_opentelemetry::layer().with_tracer(tracer);

  let filter = EnvFilter::builder()
    .with_default_directive(LevelFilter::INFO.into())
    .from_env_lossy();

  // let format = fmt::format()
  //   .with_level(false) // don't include levels in formatted output
  //   .with_target(false) // don't include targets
  //   .with_thread_ids(true) // include the thread ID of the current thread
  //   .with_thread_names(true) // include the name of the current thread
  //   .compact(); // use the `Compact` formatting style.
  let registry = Registry::default().with(filter).with(telemetry);
  if SETTINGS.json_logging {
    // tracing_subscriber::fmt()
    //   .with_env_filter(filter)
    //   .json()
    //   .init();
    registry.with(fmt::layer().json()).init();
  } else {
    registry.with(fmt::layer()).init();
  }

  // Trace executed code
  let args = CmdArgs::parse();
  start_lemmy_server(args).await?;
  Ok(())
}

// --- Traces: global tracer accessor ---
fn get_tracer() -> &'static BoxedTracer {
  static TRACER: OnceLock<BoxedTracer> = OnceLock::new();
  TRACER.get_or_init(|| global::tracer("lemmy_server"))
}

// --- Provider initialization ---
fn init_tracer_provider() -> SdkTracerProvider {
  // let provider = SdkTracerProvider::builder()
  //   .with_simple_exporter(SpanExporter::default())
  //   .build();
  // global::set_tracer_provider(provider.clone());
  // provider
  // Initialize OTLP exporter using HTTP binary protocol
  let otlp_exporter = opentelemetry_otlp::SpanExporter::builder()
    .with_http()
    .with_protocol(Protocol::HttpBinary)
    .build()
    .unwrap();

  // Create a tracer provider with the exporter
  let provider = opentelemetry_sdk::trace::SdkTracerProvider::builder()
    .with_batch_exporter(otlp_exporter)
    .build();

  // Set it as the global provider
  global::set_tracer_provider(provider.clone());
  provider
}
