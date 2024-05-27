pub mod api_routes_http;
pub mod code_migrations;
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
  middleware::{self, Condition, ErrorHandlerResponse, ErrorHandlers},
  web::Data,
  App,
  HttpResponse,
  HttpServer,
};
use actix_web_prom::PrometheusMetricsBuilder;
use clap::{Parser, Subcommand};
use lemmy_api_common::{
  context::LemmyContext,
  lemmy_db_views::structs::SiteView,
  request::client_builder,
  send_activity::{ActivityChannel, MATCH_OUTGOING_ACTIVITIES},
  utils::{
    check_private_instance_and_federation_enabled,
    local_site_rate_limit_to_rate_limit_config,
  },
};
use lemmy_apub::{
  activities::{handle_outgoing_activities, match_outgoing_activities},
  objects::instance::ApubSite,
  VerifyUrlData,
  FEDERATION_HTTP_FETCH_LIMIT,
};
use lemmy_db_schema::{schema_setup, source::secret::Secret, utils::build_db_pool};
use lemmy_federate::{start_stop_federation_workers_cancellable, Opts};
use lemmy_routes::{feeds, images, nodeinfo, webfinger};
use lemmy_utils::{
  error::LemmyResult,
  rate_limit::RateLimitCell,
  response::jsonify_plain_text_errors,
  settings::{structs::Settings, SETTINGS},
  VERSION,
};
use prometheus::default_registry;
use prometheus_metrics::serve_prometheus;
use reqwest_middleware::ClientBuilder;
use reqwest_tracing::TracingMiddleware;
use serde_json::json;
use std::{env, ops::Deref};
use tokio::signal::unix::SignalKind;
use tracing::subscriber::set_global_default;
use tracing_actix_web::TracingLogger;
use tracing_error::ErrorLayer;
use tracing_log::LogTracer;
use tracing_subscriber::{filter::Targets, layer::SubscriberExt, Layer, Registry};
use url::Url;

#[derive(Parser, Debug)]
#[command(
  version,
  about = "A link aggregator for the fediverse",
  long_about = "A link aggregator for the fediverse.\n\nThis is the Lemmy backend API server. This will connect to a PostgreSQL database, run any pending migrations and start accepting API requests."
)]
// TODO: Instead of defining individual env vars, only specify prefix once supported by clap.
//       https://github.com/clap-rs/clap/issues/3221
pub struct CmdArgs {
  /// Don't run scheduled tasks.
  ///
  /// If you are running multiple Lemmy server processes, you probably want to disable scheduled
  /// tasks on all but one of the processes, to avoid running the tasks more often than intended.
  #[arg(long, default_value_t = false, env = "LEMMY_DISABLE_SCHEDULED_TASKS")]
  disable_scheduled_tasks: bool,
  /// Disables the HTTP server.
  ///
  /// This can be used to run a Lemmy server process that only performs scheduled tasks or activity
  /// sending.
  #[arg(long, default_value_t = false, env = "LEMMY_DISABLE_HTTP_SERVER")]
  disable_http_server: bool,
  /// Disable sending outgoing ActivityPub messages.
  ///
  /// Only pass this for horizontally scaled setups.
  /// See https://join-lemmy.org/docs/administration/horizontal_scaling.html for details.
  #[arg(long, default_value_t = false, env = "LEMMY_DISABLE_ACTIVITY_SENDING")]
  disable_activity_sending: bool,
  /// The index of this outgoing federation process.
  ///
  /// Defaults to 1/1. If you want to split the federation workload onto n servers, run each server
  /// 1≤i≤n with these args: --federate-process-index i --federate-process-count n
  ///
  /// Make you have exactly one server with each `i` running, otherwise federation will randomly
  /// send duplicates or nothing.
  ///
  /// See https://join-lemmy.org/docs/administration/horizontal_scaling.html for more detail.
  #[arg(long, default_value_t = 1, env = "LEMMY_FEDERATE_PROCESS_INDEX")]
  federate_process_index: i32,
  /// How many outgoing federation processes you are starting in total.
  ///
  /// If set, make sure to set --federate-process-index differently for each.
  #[arg(long, default_value_t = 1, env = "LEMMY_FEDERATE_PROCESS_COUNT")]
  federate_process_count: i32,
  #[command(subcommand)]
  subcommand: Option<CmdSubcommand>,
}

#[derive(Subcommand, Debug)]
enum CmdSubcommand {
  /// Do something with migrations, then exit.
  Migration {
    #[command(subcommand)]
    subcommand: MigrationSubcommand,
    #[arg(long, default_value_t = false)]
    all: bool,
    #[arg(long, default_value_t = 1)]
    number: u64,
  },
}

#[derive(Subcommand, Debug)]
enum MigrationSubcommand {
  /// Run up.sql for the specified migrations.
  Run,
  /// Run down.sql for the specified migrations.
  Revert,
}

/// Placing the main function in lib.rs allows other crates to import it and embed Lemmy
pub async fn start_lemmy_server(args: CmdArgs) -> LemmyResult<()> {
  // Print version number to log
  println!("Lemmy v{VERSION}");

  if let Some(CmdSubcommand::Migration {
    subcommand,
    all,
    number,
  }) = args.subcommand
  {
    let mut options = match subcommand {
      MigrationSubcommand::Run => schema_setup::Options::default().run(),
      MigrationSubcommand::Revert => schema_setup::Options::default().revert(),
    };

    if !all {
      options = options.limit(number);
    }

    schema_setup::run(options)?;

    return Ok(());
  }

  // return error 503 while running db migrations and startup tasks
  let mut startup_server_handle = None;
  if !args.disable_http_server {
    startup_server_handle = Some(create_startup_server()?);
  }

  // Set up the connection pool
  let pool = build_db_pool().await?;

  // Run the Code-required migrations
  run_advanced_migrations(&mut (&pool).into(), &SETTINGS).await?;

  // Initialize the secrets
  let secret = Secret::init(&mut (&pool).into())
    .await?
    .expect("Couldn't initialize secrets.");

  // Make sure the local site is set up.
  let site_view = SiteView::read_local(&mut (&pool).into())
    .await?
    .expect("local site not set up");
  let local_site = site_view.local_site;
  let federation_enabled = local_site.federation_enabled;

  if federation_enabled {
    println!("Federation enabled, host is {}", &SETTINGS.hostname);
  }

  check_private_instance_and_federation_enabled(&local_site)?;

  // Set up the rate limiter
  let rate_limit_config =
    local_site_rate_limit_to_rate_limit_config(&site_view.local_site_rate_limit);
  let rate_limit_cell = RateLimitCell::new(rate_limit_config);

  println!(
    "Starting HTTP server at {}:{}",
    SETTINGS.bind, SETTINGS.port
  );

  let client = ClientBuilder::new(client_builder(&SETTINGS).build()?)
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

  if let Some(prometheus) = SETTINGS.prometheus.clone() {
    serve_prometheus(prometheus, context.clone())?;
  }

  let mut federation_config = FederationConfig::builder();
  federation_config
    .domain(SETTINGS.hostname.clone())
    .app_data(context.clone())
    .client(client.clone())
    .http_fetch_limit(FEDERATION_HTTP_FETCH_LIMIT)
    .debug(cfg!(debug_assertions))
    .http_signature_compat(true)
    .url_verifier(Box::new(VerifyUrlData(context.inner_pool().clone())));
  if local_site.federation_signed_fetch {
    let site: ApubSite = site_view.site.into();
    federation_config.signed_fetch_actor(&site);
  }
  let federation_config = federation_config.build().await?;

  MATCH_OUTGOING_ACTIVITIES
    .set(Box::new(move |d, c| {
      Box::pin(match_outgoing_activities(d, c))
    }))
    .expect("set function pointer");
  let request_data = federation_config.to_request_data();
  let outgoing_activities_task = tokio::task::spawn(handle_outgoing_activities(request_data));

  let server = if !args.disable_http_server {
    if let Some(startup_server_handle) = startup_server_handle {
      startup_server_handle.stop(true).await;
    }

    Some(create_http_server(
      federation_config.clone(),
      SETTINGS.clone(),
      federation_enabled,
    )?)
  } else {
    None
  };
  let federate = (!args.disable_activity_sending).then(|| {
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
fn create_startup_server() -> LemmyResult<ServerHandle> {
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
) -> LemmyResult<ServerHandle> {
  // this must come before the HttpServer creation
  // creates a middleware that populates http metrics for each path, method, and status code
  let prom_api_metrics = PrometheusMetricsBuilder::new("lemmy_api")
    .registry(default_registry().clone())
    .build()
    .expect("Should always be buildable");

  let context: LemmyContext = federation_config.deref().clone();
  let rate_limit_cell = federation_config.rate_limit_cell().clone();

  // Pictrs cannot use proxy
  let pictrs_client = ClientBuilder::new(client_builder(&SETTINGS).no_proxy().build()?)
    .with(TracingMiddleware::default())
    .build();

  // Create Http server
  let bind = (settings.bind, settings.port);
  let server = HttpServer::new(move || {
    let cors_config = cors_config(&settings);
    let app = App::new()
      .wrap(middleware::Logger::new(
        // This is the default log format save for the usage of %{r}a over %a to guarantee to
        // record the client's (forwarded) IP and not the last peer address, since the latter is
        // frequently just a reverse proxy
        "%{r}a '%r' %s %b '%{Referer}i' '%{User-Agent}i' %T",
      ))
      .wrap(middleware::Compress::default())
      .wrap(cors_config)
      .wrap(TracingLogger::<QuieterRootSpanBuilder>::new())
      .wrap(ErrorHandlers::new().default_handler(jsonify_plain_text_errors))
      .app_data(Data::new(context.clone()))
      .app_data(Data::new(rate_limit_cell.clone()))
      .wrap(FederationMiddleware::new(federation_config.clone()))
      .wrap(SessionMiddleware::new(context.clone()))
      .wrap(Condition::new(
        SETTINGS.prometheus.is_some(),
        prom_api_metrics.clone(),
      ));

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
  .bind(bind)?
  .run();
  let handle = server.handle();
  tokio::task::spawn(server);
  Ok(handle)
}

fn cors_config(settings: &Settings) -> Cors {
  let self_origin = settings.get_protocol_and_hostname();
  let cors_origin_setting = settings.cors_origin();
  match (cors_origin_setting.clone(), cfg!(debug_assertions)) {
    (Some(origin), false) => {
      // Need to call send_wildcard() explicitly, passing this into allowed_origin() results in
      // error
      if cors_origin_setting.as_deref() == Some("*") {
        Cors::default().allow_any_origin().send_wildcard()
      } else {
        Cors::default()
          .allowed_origin(&origin)
          .allowed_origin(&self_origin)
      }
    }
    _ => Cors::default()
      .allow_any_origin()
      .allow_any_method()
      .allow_any_header()
      .expose_any_header()
      .max_age(3600),
  }
}

pub fn init_logging(opentelemetry_url: &Option<Url>) -> LemmyResult<()> {
  LogTracer::init()?;

  let log_description = env::var("RUST_LOG").unwrap_or_else(|_| "info".into());

  let targets = log_description
    .trim()
    .trim_matches('"')
    .parse::<Targets>()?;

  let format_layer = {
    #[cfg(feature = "json-log")]
    let layer = tracing_subscriber::fmt::layer().with_ansi(false).json();
    #[cfg(not(feature = "json-log"))]
    let layer = tracing_subscriber::fmt::layer().with_ansi(false);

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
