pub mod api_routes_http;
pub mod code_migrations;
#[cfg(feature = "prometheus-metrics")]
pub mod prometheus_metrics;
pub mod root_span_builder;
pub mod scheduled_tasks;
pub mod session_middleware;
#[cfg(feature = "console")]
pub mod telemetry;

use crate::{
  code_migrations::run_advanced_migrations,
  root_span_builder::QuieterRootSpanBuilder,
  session_middleware::SessionMiddleware,
};
use activitypub_federation::config::{FederationConfig, FederationMiddleware};
use actix_cors::Cors;
use actix_web::{
  dev::{ServerHandle, ServiceResponse},
  middleware::{self, ErrorHandlerResponse, ErrorHandlers},
  web::Data,
  App,
  HttpResponse,
  HttpServer,
  Result,
};
use clap::{ArgAction, Parser};
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
use lemmy_federate::{start_stop_federation_workers_cancellable, Opts};
use lemmy_routes::{feeds, images, nodeinfo, webfinger};
use lemmy_utils::{
  error::LemmyError,
  rate_limit::RateLimitCell,
  response::jsonify_plain_text_errors,
  settings::{structs::Settings, SETTINGS},
};
use reqwest::Client;
use reqwest_middleware::{ClientBuilder, ClientWithMiddleware};
use reqwest_tracing::TracingMiddleware;
use serde_json::json;
use std::{env, ops::Deref, time::Duration};
use tokio::signal::unix::SignalKind;
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

#[derive(Parser, Debug)]
#[command(
  version,
  about = "A link aggregator for the fediverse",
  long_about = "A link aggregator for the fediverse.\n\nThis is the Lemmy backend API server. This will connect to a PostgreSQL database, run any pending migrations and start accepting API requests."
)]
pub struct CmdArgs {
  #[arg(long, default_value_t = false)]
  /// Disables running scheduled tasks.
  ///
  /// If you are running multiple Lemmy server processes,
  /// you probably want to disable scheduled tasks on all but one of the processes,
  /// to avoid running the tasks more often than intended.
  disable_scheduled_tasks: bool,
  /// Whether or not to run the HTTP server.
  ///
  /// This can be used to run a Lemmy server process that only runs scheduled tasks.
  #[arg(long, default_value_t = true, action=ArgAction::Set)]
  http_server: bool,
  /// Whether or not to emit outgoing ActivityPub messages.
  ///
  /// Set to true for a simple setup. Only set to false for horizontally scaled setups.
  /// See https://join-lemmy.org/docs/administration/horizontal_scaling.html for detail.
  #[arg(long, default_value_t = true, action=ArgAction::Set)]
  federate_activities: bool,
  /// The index of this outgoing federation process.
  ///
  /// Defaults to 1/1. If you want to split the federation workload onto n servers, run each server 1≤i≤n with these args:
  /// --federate-process-index i --federate-process-count n
  ///
  /// Make you have exactly one server with each `i` running, otherwise federation will randomly send duplicates or nothing.
  ///
  /// See https://join-lemmy.org/docs/administration/horizontal_scaling.html for more detail.
  #[arg(long, default_value_t = 1)]
  federate_process_index: i32,
  /// How many outgoing federation processes you are starting in total.
  ///
  /// If set, make sure to set --federate-process-index differently for each.
  #[arg(long, default_value_t = 1)]
  federate_process_count: i32,
}
/// Max timeout for http requests
pub(crate) const REQWEST_TIMEOUT: Duration = Duration::from_secs(10);

/// Placing the main function in lib.rs allows other crates to import it and embed Lemmy
pub async fn start_lemmy_server(args: CmdArgs) -> Result<(), LemmyError> {
  let settings = SETTINGS.to_owned();

  // return error 503 while running db migrations and startup tasks
  let mut startup_server_handle = None;
  if args.http_server {
    startup_server_handle = Some(create_startup_server()?);
  }

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

  if !args.disable_scheduled_tasks {
    // Schedules various cleanup tasks for the DB
    let _scheduled_tasks = tokio::task::spawn(scheduled_tasks::setup(context.clone()));
  }

  #[cfg(feature = "prometheus-metrics")]
  serve_prometheus(settings.prometheus.as_ref(), context.clone());

  let federation_config = FederationConfig::builder()
    .domain(settings.hostname.clone())
    .app_data(context.clone())
    .client(client.clone())
    .http_fetch_limit(FEDERATION_HTTP_FETCH_LIMIT)
    .debug(cfg!(debug_assertions))
    .http_signature_compat(true)
    .url_verifier(Box::new(VerifyUrlData(context.inner_pool().clone())))
    .build()
    .await?;

  MATCH_OUTGOING_ACTIVITIES
    .set(Box::new(move |d, c| {
      Box::pin(match_outgoing_activities(d, c))
    }))
    .expect("set function pointer");
  let request_data = federation_config.to_request_data();
  let outgoing_activities_task = tokio::task::spawn(handle_outgoing_activities(request_data));

  let server = if args.http_server {
    if let Some(startup_server_handle) = startup_server_handle {
      startup_server_handle.stop(true).await;
    }
    Some(create_http_server(
      federation_config.clone(),
      settings.clone(),
      federation_enabled,
      pictrs_client,
    )?)
  } else {
    None
  };
  let federate = args.federate_activities.then(|| {
    start_stop_federation_workers_cancellable(
      Opts {
        process_index: args.federate_process_index,
        process_count: args.federate_process_count,
      },
      pool.clone(),
      federation_config.clone(),
    )
  });
  let mut interrupt = tokio::signal::unix::signal(SignalKind::interrupt())?;
  let mut terminate = tokio::signal::unix::signal(SignalKind::terminate())?;

  tokio::select! {
    _ = tokio::signal::ctrl_c() => {
      tracing::warn!("Received ctrl-c, shutting down gracefully...");
    }
    _ = interrupt.recv() => {
      tracing::warn!("Received interrupt, shutting down gracefully...");
    }
    _ = terminate.recv() => {
      tracing::warn!("Received terminate, shutting down gracefully...");
    }
  }
  if let Some(server) = server {
    server.stop(true).await;
  }
  if let Some(federate) = federate {
    federate.cancel().await?;
  }

  // Wait for outgoing apub sends to complete
  ActivityChannel::close(outgoing_activities_task).await?;

  Ok(())
}

/// Creates temporary HTTP server which returns status 503 for all requests.
fn create_startup_server() -> Result<ServerHandle, LemmyError> {
  let startup_server = HttpServer::new(move || {
    App::new().wrap(ErrorHandlers::new().default_handler(move |req| {
      let (req, _) = req.into_parts();
      let response =
        HttpResponse::ServiceUnavailable().json(json!({"error": "Lemmy is currently starting"}));
      let service_response = ServiceResponse::new(req, response);
      Ok(ErrorHandlerResponse::Response(
        service_response.map_into_right_body(),
      ))
    }))
  })
  .bind((SETTINGS.bind, SETTINGS.port))?
  .run();
  let startup_server_handle = startup_server.handle();
  tokio::task::spawn(startup_server);
  Ok(startup_server_handle)
}

fn create_http_server(
  federation_config: FederationConfig<LemmyContext>,
  settings: Settings,
  federation_enabled: bool,
  pictrs_client: ClientWithMiddleware,
) -> Result<ServerHandle, LemmyError> {
  // this must come before the HttpServer creation
  // creates a middleware that populates http metrics for each path, method, and status code
  #[cfg(feature = "prometheus-metrics")]
  let prom_api_metrics = PrometheusMetricsBuilder::new("lemmy_api")
    .registry(default_registry().clone())
    .build()
    .expect("Should always be buildable");

  let context: LemmyContext = federation_config.deref().clone();
  let rate_limit_cell = federation_config.rate_limit_cell().clone();
  let self_origin = settings.get_protocol_and_hostname();
  // Create Http server with websocket support
  let server = HttpServer::new(move || {
    let cors_origin = env::var("LEMMY_CORS_ORIGIN");
    let cors_config = match (cors_origin, cfg!(debug_assertions)) {
      (Ok(origin), false) => Cors::default()
        .allowed_origin(&origin)
        .allowed_origin(&self_origin),
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
      .wrap(FederationMiddleware::new(federation_config.clone()))
      .wrap(SessionMiddleware::new(context.clone()));

    #[cfg(feature = "prometheus-metrics")]
    let app = app.wrap(prom_api_metrics.clone());

    // The routes
    app
      .configure(|cfg| api_routes_http::config(cfg, &rate_limit_cell))
      .configure(|cfg| {
        if federation_enabled {
          lemmy_apub::http::routes::config(cfg);
          webfinger::config(cfg);
        }
      })
      .configure(feeds::config)
      .configure(|cfg| images::config(cfg, pictrs_client.clone(), &rate_limit_cell))
      .configure(nodeinfo::config)
  })
  .disable_signals()
  .bind((settings.bind, settings.port))?
  .run();
  let handle = server.handle();
  tokio::task::spawn(server);
  Ok(handle)
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
