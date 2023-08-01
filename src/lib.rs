pub mod api_routes_http;
pub mod code_migrations;
#[cfg(feature = "prometheus-metrics")]
pub mod prometheus_metrics;
pub mod root_span_builder;
pub mod scheduled_tasks;
#[cfg(feature = "console")]
pub mod telemetry;

use crate::{code_migrations::run_advanced_migrations, root_span_builder::QuieterRootSpanBuilder};
use activitypub_federation::config::{FederationConfig, FederationMiddleware};
use actix_cors::Cors;
use actix_web::{
  middleware::{self, ErrorHandlers},
  web::Data,
  App,
  HttpServer,
  Result,
};
use lemmy_api_common::{
  context::LemmyContext,
  lemmy_db_views::structs::SiteView,
  request::build_user_agent,
  send_activity::{ActivityChannel, MATCH_OUTGOING_ACTIVITIES},
  utils::{
    check_private_instance_and_federation_enabled,
    local_site_rate_limit_to_rate_limit_config,
  },
};
use lemmy_apub::{
  activities::{handle_outgoing_activities, match_outgoing_activities},
  VerifyUrlData,
  FEDERATION_HTTP_FETCH_LIMIT,
};
use lemmy_db_schema::{
  source::secret::Secret,
  utils::{build_db_pool, get_database_url, run_migrations},
};
use lemmy_routes::{feeds, images, nodeinfo, webfinger};
use lemmy_utils::{
  error::LemmyError,
  rate_limit::RateLimitCell,
  response::jsonify_plain_text_errors,
  settings::SETTINGS,
  SYNCHRONOUS_FEDERATION,
};
use reqwest::Client;
use reqwest_middleware::ClientBuilder;
use reqwest_tracing::TracingMiddleware;
use std::{env, thread, time::Duration};
use tracing::subscriber::set_global_default;
use tracing_actix_web::TracingLogger;
use tracing_error::ErrorLayer;
use tracing_log::LogTracer;
use tracing_subscriber::{filter::Targets, layer::SubscriberExt, Layer, Registry};
use url::Url;
#[cfg(feature = "prometheus-metrics")]
use {
  actix_web_prom::PrometheusMetricsBuilder,
  prometheus::default_registry,
  prometheus_metrics::serve_prometheus,
};

/// Max timeout for http requests
pub(crate) const REQWEST_TIMEOUT: Duration = Duration::from_secs(10);

/// Placing the main function in lib.rs allows other crates to import it and embed Lemmy
pub async fn start_lemmy_server() -> Result<(), LemmyError> {
  println!("test");
  let args: Vec<String> = env::args().collect();

  let scheduled_tasks_enabled = args.get(1) != Some(&"--disable-scheduled-tasks".to_string());

  let settings = SETTINGS.to_owned();

  // Run the DB migrations
  let db_url = get_database_url(Some(&settings));
  run_migrations(&db_url);

  // Set up the connection pool
  let pool = build_db_pool(&settings).await?;

  // Run the Code-required migrations
  run_advanced_migrations(&mut (&pool).into(), &settings).await?;

  // Initialize the secrets
  let secret = Secret::init(&mut (&pool).into())
    .await
    .expect("Couldn't initialize secrets.");

  // Make sure the local site is set up.
  let site_view = SiteView::read_local(&mut (&pool).into())
    .await
    .expect("local site not set up");
  let local_site = site_view.local_site;
  let federation_enabled = local_site.federation_enabled;

  if federation_enabled {
    println!("federation enabled, host is {}", &settings.hostname);
  }

  check_private_instance_and_federation_enabled(&local_site)?;

  // Set up the rate limiter
  let rate_limit_config =
    local_site_rate_limit_to_rate_limit_config(&site_view.local_site_rate_limit);
  let rate_limit_cell = RateLimitCell::new(rate_limit_config).await;

  println!(
    "Starting http server at {}:{}",
    settings.bind, settings.port
  );

  let user_agent = build_user_agent(&settings);
  let reqwest_client = Client::builder()
    .user_agent(user_agent.clone())
    .timeout(REQWEST_TIMEOUT)
    .connect_timeout(REQWEST_TIMEOUT)
    .build()?;

  let client = ClientBuilder::new(reqwest_client.clone())
    .with(TracingMiddleware::default())
    .build();

  // Pictrs cannot use the retry middleware
  let pictrs_client = ClientBuilder::new(reqwest_client.clone())
    .with(TracingMiddleware::default())
    .build();

  let context = LemmyContext::create(
    pool.clone(),
    client.clone(),
    secret.clone(),
    rate_limit_cell.clone(),
  );

  if scheduled_tasks_enabled {
    // Schedules various cleanup tasks for the DB
    thread::spawn({
      let context = context.clone();
      move || {
        scheduled_tasks::setup(db_url, user_agent, context)
          .expect("Couldn't set up scheduled_tasks");
      }
    });
  }

  #[cfg(feature = "prometheus-metrics")]
  serve_prometheus(settings.prometheus.as_ref(), context.clone());

  let settings_bind = settings.clone();

  let federation_config = FederationConfig::builder()
    .domain(settings.hostname.clone())
    .app_data(context.clone())
    .client(client.clone())
    .http_fetch_limit(FEDERATION_HTTP_FETCH_LIMIT)
    .worker_count(settings.worker_count)
    .retry_count(settings.retry_count)
    .debug(*SYNCHRONOUS_FEDERATION)
    .http_signature_compat(true)
    .url_verifier(Box::new(VerifyUrlData(context.inner_pool().clone())))
    .build()
    .await?;

  // this must come before the HttpServer creation
  // creates a middleware that populates http metrics for each path, method, and status code
  #[cfg(feature = "prometheus-metrics")]
  let prom_api_metrics = PrometheusMetricsBuilder::new("lemmy_api")
    .registry(default_registry().clone())
    .build()
    .expect("Should always be buildable");

  MATCH_OUTGOING_ACTIVITIES
    .set(Box::new(move |d, c| {
      Box::pin(match_outgoing_activities(d, c))
    }))
    .expect("set function pointer");
  let request_data = federation_config.to_request_data();
  let outgoing_activities_task = tokio::task::spawn(handle_outgoing_activities(request_data));

  // Create Http server with websocket support
  HttpServer::new(move || {
    let cors_origin = env::var("LEMMY_CORS_ORIGIN");
    let cors_config = match (cors_origin, cfg!(debug_assertions)) {
      (Ok(origin), false) => Cors::default()
        .allowed_origin(&origin)
        .allowed_origin(&settings.get_protocol_and_hostname()),
      _ => Cors::default()
        .allow_any_origin()
        .allow_any_method()
        .allow_any_header()
        .expose_any_header()
        .max_age(3600),
    };

    let app = App::new()
      .wrap(middleware::Logger::new(
        // This is the default log format save for the usage of %{r}a over %a to guarantee to record the client's (forwarded) IP and not the last peer address, since the latter is frequently just a reverse proxy
        "%{r}a '%r' %s %b '%{Referer}i' '%{User-Agent}i' %T",
      ))
      .wrap(middleware::Compress::default())
      .wrap(cors_config)
      .wrap(TracingLogger::<QuieterRootSpanBuilder>::new())
      .wrap(ErrorHandlers::new().default_handler(jsonify_plain_text_errors))
      .app_data(Data::new(context.clone()))
      .app_data(Data::new(rate_limit_cell.clone()))
      .wrap(FederationMiddleware::new(federation_config.clone()));

    #[cfg(feature = "prometheus-metrics")]
    let app = app.wrap(prom_api_metrics.clone());

    // The routes
    app
      .configure(|cfg| api_routes_http::config(cfg, rate_limit_cell))
      .configure(|cfg| {
        if federation_enabled {
          lemmy_apub::http::routes::config(cfg);
          webfinger::config(cfg);
        }
      })
      .configure(feeds::config)
      .configure(|cfg| images::config(cfg, pictrs_client.clone(), rate_limit_cell))
      .configure(nodeinfo::config)
  })
  .bind((settings_bind.bind, settings_bind.port))?
  .run()
  .await?;

  // Wait for outgoing apub sends to complete
  ActivityChannel::close(outgoing_activities_task).await?;

  Ok(())
}

pub fn init_logging(opentelemetry_url: &Option<Url>) -> Result<(), LemmyError> {
  LogTracer::init()?;

  let log_description = std::env::var("RUST_LOG").unwrap_or_else(|_| "info".into());

  let targets = log_description
    .trim()
    .trim_matches('"')
    .parse::<Targets>()?;

  let format_layer = {
    #[cfg(feature = "json-log")]
    let layer = tracing_subscriber::fmt::layer().json();
    #[cfg(not(feature = "json-log"))]
    let layer = tracing_subscriber::fmt::layer();

    layer.with_filter(targets.clone())
  };

  let subscriber = Registry::default()
    .with(format_layer)
    .with(ErrorLayer::default());

  if let Some(_url) = opentelemetry_url {
    #[cfg(feature = "console")]
    telemetry::init_tracing(_url.as_ref(), subscriber, targets)?;
    #[cfg(not(feature = "console"))]
    tracing::error!("Feature `console` must be enabled for opentelemetry tracing");
  } else {
    set_global_default(subscriber)?;
  }

  Ok(())
}
