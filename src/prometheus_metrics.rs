use actix_web::{rt::System, web, App, HttpServer};
use lemmy_api_common::context::LemmyContext;
use lemmy_utils::{error::LemmyResult, settings::structs::PrometheusConfig};
use prometheus::{default_registry, Encoder, Gauge, Opts, TextEncoder};
use std::{sync::Arc, thread};
use tracing::error;

struct PromContext {
  lemmy: LemmyContext,
  db_pool_metrics: DbPoolMetrics,
}

struct DbPoolMetrics {
  max_size: Gauge,
  size: Gauge,
  available: Gauge,
}

pub fn serve_prometheus(config: PrometheusConfig, lemmy_context: LemmyContext) -> LemmyResult<()> {
  let context = Arc::new(PromContext {
    lemmy: lemmy_context,
    db_pool_metrics: create_db_pool_metrics()?,
  });

  // spawn thread that blocks on handling requests
  // only mapping /metrics to a handler
  thread::spawn(move || {
    let sys = System::new();
    sys.block_on(async {
      let server = HttpServer::new(move || {
        App::new()
          .app_data(web::Data::new(Arc::clone(&context)))
          .route("/metrics", web::get().to(metrics))
      })
      .bind((config.bind, config.port as u16))
      .unwrap_or_else(|e| panic!("Cannot bind to {}:{}: {e}", config.bind, config.port))
      .run();

      if let Err(err) = server.await {
        error!("Prometheus server error: {err}");
      }
    })
  });
  Ok(())
}

// handler for the /metrics path
async fn metrics(context: web::Data<Arc<PromContext>>) -> LemmyResult<String> {
  // collect metrics
  collect_db_pool_metrics(&context).await;

  let mut buffer = Vec::new();
  let encoder = TextEncoder::new();

  // gather metrics from registry and encode in prometheus format
  let metric_families = prometheus::gather();
  encoder.encode(&metric_families, &mut buffer)?;
  let output = String::from_utf8(buffer)?;

  Ok(output)
}

// create lemmy_db_pool_* metrics and register them with the default registry
fn create_db_pool_metrics() -> LemmyResult<DbPoolMetrics> {
  let metrics = DbPoolMetrics {
    max_size: Gauge::with_opts(Opts::new(
      "lemmy_db_pool_max_connections",
      "Maximum number of connections in the pool",
    ))?,
    size: Gauge::with_opts(Opts::new(
      "lemmy_db_pool_connections",
      "Current number of connections in the pool",
    ))?,
    available: Gauge::with_opts(Opts::new(
      "lemmy_db_pool_available_connections",
      "Number of available connections in the pool",
    ))?,
  };

  default_registry().register(Box::new(metrics.max_size.clone()))?;
  default_registry().register(Box::new(metrics.size.clone()))?;
  default_registry().register(Box::new(metrics.available.clone()))?;

  Ok(metrics)
}

async fn collect_db_pool_metrics(context: &PromContext) {
  let pool_status = context.lemmy.inner_pool().status();
  context
    .db_pool_metrics
    .max_size
    .set(pool_status.max_size as f64);
  context.db_pool_metrics.size.set(pool_status.size as f64);
  context
    .db_pool_metrics
    .available
    .set(pool_status.available as f64);
}
