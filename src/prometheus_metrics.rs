// TODO: should really not unwrap everywhere here....
#![allow(clippy::unwrap_used)]
use actix_web::{rt::System, web, App, HttpResponse, HttpServer, Responder};
use lemmy_api_common::context::LemmyContext;
use lemmy_utils::settings::structs::PrometheusConfig;
use prometheus::{default_registry, Encoder, Gauge, Opts, TextEncoder};
use std::{
  net::{IpAddr, Ipv4Addr},
  sync::Arc,
  thread,
};

struct PromContext {
  lemmy: LemmyContext,
  db_pool_metrics: DbPoolMetrics,
}

struct DbPoolMetrics {
  max_size: Gauge,
  size: Gauge,
  available: Gauge,
}

static DEFAULT_BIND: IpAddr = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));
static DEFAULT_PORT: i32 = 10002;

pub fn serve_prometheus(config: Option<&PrometheusConfig>, lemmy_context: LemmyContext) {
  let context = Arc::new(PromContext {
    lemmy: lemmy_context,
    db_pool_metrics: create_db_pool_metrics(),
  });

  let (bind, port) = match config {
    Some(config) => (
      config.bind.unwrap_or(DEFAULT_BIND),
      config.port.unwrap_or(DEFAULT_PORT),
    ),
    None => (DEFAULT_BIND, DEFAULT_PORT),
  };

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
      .bind((bind, port as u16))
      .unwrap_or_else(|_| panic!("Cannot bind to {}:{}", bind, port))
      .run();

      if let Err(err) = server.await {
        eprintln!("Prometheus server error: {}", err);
      }
    })
  });
}

// handler for the /metrics path
async fn metrics(context: web::Data<Arc<PromContext>>) -> impl Responder {
  // collect metrics
  collect_db_pool_metrics(&context).await;

  let mut buffer = Vec::new();
  let encoder = TextEncoder::new();

  // gather metrics from registry and encode in prometheus format
  let metric_families = prometheus::gather();
  encoder.encode(&metric_families, &mut buffer).unwrap();
  let output = String::from_utf8(buffer).unwrap();

  HttpResponse::Ok().body(output)
}

// create lemmy_db_pool_* metrics and register them with the default registry
fn create_db_pool_metrics() -> DbPoolMetrics {
  let metrics = DbPoolMetrics {
    max_size: Gauge::with_opts(Opts::new(
      "lemmy_db_pool_max_connections",
      "Maximum number of connections in the pool",
    ))
    .unwrap(),
    size: Gauge::with_opts(Opts::new(
      "lemmy_db_pool_connections",
      "Current number of connections in the pool",
    ))
    .unwrap(),
    available: Gauge::with_opts(Opts::new(
      "lemmy_db_pool_available_connections",
      "Number of available connections in the pool",
    ))
    .unwrap(),
  };

  default_registry()
    .register(Box::new(metrics.max_size.clone()))
    .unwrap();
  default_registry()
    .register(Box::new(metrics.size.clone()))
    .unwrap();
  default_registry()
    .register(Box::new(metrics.available.clone()))
    .unwrap();

  metrics
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
