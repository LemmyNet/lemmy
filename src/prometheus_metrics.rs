use actix_web::{rt::System, web, App, HttpResponse, HttpServer, Responder};
use lemmy_api_common::context::LemmyContext;
use lemmy_utils::settings::structs::PrometheusConfig;
use prometheus::{core::Collector, default_registry, Encoder, Gauge, Opts, TextEncoder};
use std::{
  net::{IpAddr, Ipv4Addr},
  sync::Arc,
  thread,
};

struct PromContext {
  lemmy: LemmyContext,
}

struct DbPoolMetrics {
  context: Arc<PromContext>,
  size: Gauge,
  max_size: Gauge,
  available: Gauge,
}

static DEFAULT_BIND: IpAddr = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));
static DEFAULT_PORT: i32 = 10002;

pub fn serve_prometheus_metrics(config: Option<&PrometheusConfig>, lemmy_context: LemmyContext) {
  let context = Arc::new(PromContext {
    lemmy: lemmy_context,
  });

  // register custom collectors
  default_registry()
    .register(Box::new(DbPoolMetrics::new(Arc::clone(&context))))
    .expect("Failed to register DbPoolMetrics");

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
      .unwrap_or_else(|_| panic!("Prometheus server cannot bind to {}:{}", bind, port))
      .run();

      if let Err(err) = server.await {
        eprintln!("Prometheus server error: {}", err);
      }
    })
  });
}

// handler for the /metrics path
async fn metrics(_context: web::Data<Arc<PromContext>>) -> impl Responder {
  let mut buffer = Vec::new();
  let encoder = TextEncoder::new();

  // gather metrics from registry
  let metric_families = prometheus::gather();

  // encode in prometheus format
  if let Err(err) = encoder.encode(&metric_families, &mut buffer) {
    eprintln!("Prometheus encoding error: {}", err);
    return HttpResponse::InternalServerError().finish();
  }

  // convert to utf-8 and return
  match String::from_utf8(buffer) {
    Ok(body) => HttpResponse::Ok().body(body),
    Err(err) => {
      eprintln!("Prometheus utf-8 encoding error: {}", err);
      HttpResponse::InternalServerError().finish()
    }
  }
}

impl DbPoolMetrics {
  fn new(context: Arc<PromContext>) -> Self {
    let size = Gauge::with_opts(Opts::new(
      "lemmy_db_pool_connections",
      "Current number of connections in the pool",
    ))
    .expect("Prometheus DbPoolMetrics: failed to create size metric");

    let max_size = Gauge::with_opts(Opts::new(
      "lemmy_db_pool_max_connections",
      "Maximum number of connections in the pool",
    ))
    .expect("Prometheus DbPoolMetrics: failed to create max_size metric");

    let available = Gauge::with_opts(Opts::new(
      "lemmy_db_pool_available_connections",
      "Number of available connections in the pool",
    ))
    .expect("Prometheus DbPoolMetrics: failed to create available metric");

    Self {
      context,
      size,
      max_size,
      available,
    }
  }
}

impl Collector for DbPoolMetrics {
  fn desc(&self) -> Vec<&prometheus::core::Desc> {
    let mut desc = vec![];
    desc.extend(self.size.desc());
    desc.extend(self.max_size.desc());
    desc.extend(self.available.desc());
    desc
  }

  fn collect(&self) -> Vec<prometheus::proto::MetricFamily> {
    let pool_status = self.context.lemmy.inner_pool().status();
    self.size.set(pool_status.size as f64);
    self.max_size.set(pool_status.max_size as f64);
    self.available.set(pool_status.available as f64);

    let mut metrics = vec![];
    metrics.extend(self.size.collect());
    metrics.extend(self.max_size.collect());
    metrics.extend(self.available.collect());
    metrics
  }
}
