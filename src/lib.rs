pub mod api_routes;

use activitypub_federation::config::{FederationConfig, FederationMiddleware};
use actix_web::{
  dev::{ServerHandle, ServiceResponse},
  middleware::{self, Condition, ErrorHandlerResponse, ErrorHandlers},
  web::{get, scope, Data},
  App,
  HttpResponse,
  HttpServer,
};
use clap::{Parser, Subcommand};
use lemmy_api::sitemap::get_sitemap;
use lemmy_api_utils::{
  context::LemmyContext,
  request::client_builder,
  send_activity::{ActivityChannel, MATCH_OUTGOING_ACTIVITIES},
  utils::local_site_rate_limit_to_rate_limit_config,
};
use lemmy_apub::{
  activities::{handle_outgoing_activities, match_outgoing_activities},
  collections::fetch_community_collections,
  VerifyUrlData,
  FEDERATION_HTTP_FETCH_LIMIT,
};
use lemmy_apub_objects::objects::{community::FETCH_COMMUNITY_COLLECTIONS, instance::ApubSite};
use lemmy_db_schema::{source::secret::Secret, utils::build_db_pool};
use lemmy_db_schema_file::schema_setup;
use lemmy_federate::{Opts, SendManager};
use lemmy_routes::{
  feeds,
  middleware::{
    idempotency::{IdempotencyMiddleware, IdempotencySet},
    session::SessionMiddleware,
  },
  nodeinfo,
  utils::{
    cors_config,
    prometheus_metrics::{new_prometheus_metrics, serve_prometheus},
    scheduled_tasks,
    setup_local_site::setup_local_site,
  },
  webfinger,
};
use lemmy_utils::{
  error::{LemmyErrorType, LemmyResult},
  rate_limit::RateLimitCell,
  response::jsonify_plain_text_errors,
  settings::{structs::Settings, SETTINGS},
  VERSION,
};
use mimalloc::MiMalloc;
use reqwest_middleware::ClientBuilder;
use reqwest_tracing::TracingMiddleware;
use serde_json::json;
use std::{ops::Deref, time::Duration};
use tokio::signal::unix::SignalKind;
use tracing_actix_web::{DefaultRootSpanBuilder, TracingLogger};

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

/// Timeout for HTTP requests while sending activities. A longer timeout provides better
/// compatibility with other ActivityPub software that might allocate more time for synchronous
/// processing of incoming activities. This timeout should be slightly longer than the time we
/// expect a remote server to wait before aborting processing on its own to account for delays from
/// establishing the HTTP connection and sending the request itself.
const ACTIVITY_SENDING_TIMEOUT: Duration = Duration::from_secs(125);

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
    /// Stop after there's no remaining migrations.
    #[arg(long, default_value_t = false)]
    all: bool,
    /// Stop after the given number of migrations.
    #[arg(long, default_value_t = 1)]
    number: u64,
  },
}

#[derive(Subcommand, Debug)]
enum MigrationSubcommand {
  /// Run up.sql for pending migrations, oldest to newest.
  Run,
  /// Run down.sql for non-pending migrations, newest to oldest.
  Revert,
}

/// Placing the main function in lib.rs allows other crates to import it and embed Lemmy
pub async fn start_lemmy_server(args: CmdArgs) -> LemmyResult<()> {
  if let Some(CmdSubcommand::Migration {
    subcommand,
    all,
    number,
  }) = args.subcommand
  {
    let mut options = match subcommand {
      MigrationSubcommand::Run => schema_setup::Options::default().run(),
      MigrationSubcommand::Revert => schema_setup::Options::default().revert(),
    }
    .print_output();

    if !all {
      options = options.limit(number);
    }

    schema_setup::run(options)?;

    return Ok(());
  }

  // Print version number to log
  println!("Starting Lemmy v{VERSION}");

  // return error 503 while running db migrations and startup tasks
  let mut startup_server_handle = None;
  if !args.disable_http_server {
    startup_server_handle = Some(create_startup_server()?);
  }

  // Set up the connection pool
  let pool = build_db_pool()?;

  // Initialize the secrets
  let secret = Secret::init(&mut (&pool).into()).await?;

  // Make sure the local site is set up.
  let site_view = setup_local_site(&mut (&pool).into(), &SETTINGS).await?;
  let federation_enabled = site_view.local_site.federation_enabled;

  if federation_enabled {
    println!("Federation enabled, host is {}", &SETTINGS.hostname);
  }

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
  let pictrs_client = ClientBuilder::new(client_builder(&SETTINGS).no_proxy().build()?)
    .with(TracingMiddleware::default())
    .build();
  let context = LemmyContext::create(
    pool.clone(),
    client.clone(),
    pictrs_client,
    secret.clone(),
    rate_limit_cell,
  );

  if let Some(prometheus) = SETTINGS.prometheus.clone() {
    serve_prometheus(prometheus, context.clone())?;
  }

  let mut federation_config_builder = FederationConfig::builder();
  federation_config_builder
    .domain(SETTINGS.hostname.clone())
    .app_data(context.clone())
    .client(client.clone())
    .http_fetch_limit(FEDERATION_HTTP_FETCH_LIMIT)
    .debug(cfg!(debug_assertions))
    .http_signature_compat(true)
    .url_verifier(Box::new(VerifyUrlData(context.inner_pool().clone())));
  if site_view.local_site.federation_signed_fetch {
    let site: ApubSite = site_view.site.clone().into();
    federation_config_builder.signed_fetch_actor(&site);
  }
  let federation_config = federation_config_builder.build().await?;

  MATCH_OUTGOING_ACTIVITIES
    .set(Box::new(move |d, c| {
      Box::pin(match_outgoing_activities(d, c))
    }))
    .map_err(|_e| LemmyErrorType::Unknown("couldnt set function pointer".into()))?;
  FETCH_COMMUNITY_COLLECTIONS
    .set(fetch_community_collections)
    .map_err(|_e| LemmyErrorType::Unknown("couldnt set function pointer".into()))?;

  let request_data = federation_config.to_request_data();
  let outgoing_activities_task = tokio::task::spawn(handle_outgoing_activities(
    request_data.reset_request_count(),
  ));

  if !args.disable_scheduled_tasks {
    // Schedules various cleanup tasks for the DB
    let _scheduled_tasks =
      tokio::task::spawn(scheduled_tasks::setup(request_data.reset_request_count()));
  }

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

  // This FederationConfig instance is exclusively used to send activities, so we can safely
  // increase the timeout without affecting timeouts for resolving objects anywhere.
  let federation_sender_config = if !args.disable_activity_sending {
    let mut federation_sender_config = federation_config_builder.clone();
    federation_sender_config.request_timeout(ACTIVITY_SENDING_TIMEOUT);
    Some(federation_sender_config.build().await?)
  } else {
    None
  };
  let federate = federation_sender_config.map(|cfg| {
    SendManager::run(
      Opts {
        process_index: args.federate_process_index,
        process_count: args.federate_process_count,
      },
      cfg,
      SETTINGS.federation.clone(),
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
  // These must come before HttpServer creation so they can collect data across threads.
  let prom_api_metrics = new_prometheus_metrics()?;
  let idempotency_set = IdempotencySet::default();

  // Create Http server
  let bind = (settings.bind, settings.port);
  let server = HttpServer::new(move || {
    let context: LemmyContext = federation_config.deref().clone();
    let rate_limit_cell = federation_config.rate_limit_cell().clone();

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
      .wrap(TracingLogger::<DefaultRootSpanBuilder>::new())
      .wrap(ErrorHandlers::new().default_handler(jsonify_plain_text_errors))
      .app_data(Data::new(context.clone()))
      .wrap(FederationMiddleware::new(federation_config.clone()))
      .wrap(IdempotencyMiddleware::new(idempotency_set.clone()))
      .wrap(SessionMiddleware::new(context.clone()))
      .wrap(Condition::new(
        SETTINGS.prometheus.is_some(),
        prom_api_metrics.clone(),
      ));

    // The routes
    app
      .configure(|cfg| api_routes::config(cfg, &rate_limit_cell))
      .configure(|cfg| {
        if federation_enabled {
          lemmy_apub::http::routes::config(cfg);
          webfinger::config(cfg);
        }
      })
      .configure(feeds::config)
      .configure(nodeinfo::config)
      .service(
        scope("/sitemap.xml")
          .wrap(rate_limit_cell.message())
          .route("", get().to(get_sitemap)),
      )
  })
  .disable_signals()
  .bind(bind)?
  .run();
  let handle = server.handle();
  tokio::task::spawn(server);
  Ok(handle)
}
