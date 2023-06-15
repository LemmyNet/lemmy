pub mod api_routes_http;
pub mod code_migrations;
pub mod root_span_builder;
pub mod scheduled_tasks;
#[cfg(feature = "console")]
pub mod telemetry;

use crate::{code_migrations::run_advanced_migrations, root_span_builder::QuieterRootSpanBuilder};
use activitypub_federation::config::{FederationConfig, FederationMiddleware};
use actix_cors::Cors;
use actix_web::{middleware, web::Data, App, HttpServer, Result};
use doku::json::{AutoComments, CommentsStyle, Formatting, ObjectsStyle};
use lemmy_api_common::{
  context::LemmyContext,
  lemmy_db_views::structs::SiteView,
  request::build_user_agent,
  utils::{
    check_private_instance_and_federation_enabled,
    local_site_rate_limit_to_rate_limit_config,
  },
};
use lemmy_apub::{VerifyUrlData, FEDERATION_HTTP_FETCH_LIMIT};
use lemmy_db_schema::{
  source::secret::Secret,
  utils::{build_db_pool, get_database_url, run_migrations},
};
use lemmy_routes::{feeds, images, nodeinfo, webfinger};
use lemmy_utils::{
  error::LemmyError,
  rate_limit::RateLimitCell,
  settings::{structs::Settings, SETTINGS},
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

/// Max timeout for http requests
pub(crate) const REQWEST_TIMEOUT: Duration = Duration::from_secs(10);

/// Placing the main function in lib.rs allows other crates to import it and embed Lemmy
pub async fn start_lemmy_server() -> Result<(), LemmyError> {
  let args: Vec<String> = env::args().collect();
  if args.get(1) == Some(&"--print-config-docs".to_string()) {
    let fmt = Formatting {
      auto_comments: AutoComments::none(),
      comments_style: CommentsStyle {
        separator: "#".to_owned(),
      },
      objects_style: ObjectsStyle {
        surround_keys_with_quotes: false,
        use_comma_as_separator: false,
      },
      ..Default::default()
    };
    println!("{}", doku::to_json_fmt_val(&fmt, &Settings::default()));
    return Ok(());
  }

  let scheduled_tasks_enabled = args.get(1) != Some(&"--disable-scheduled-tasks".to_string());

  let settings = SETTINGS.to_owned();

  // Run the DB migrations
  let db_url = get_database_url(Some(&settings));
  run_migrations(&db_url);

  // Set up the connection pool
  let pool = build_db_pool(&settings).await?;

  // Run the Code-required migrations
  run_advanced_migrations(&pool, &settings).await?;

  // Initialize the secrets
  let secret = Secret::init(&pool)
    .await
    .expect("Couldn't initialize secrets.");

  // Make sure the local site is set up.
  let site_view = SiteView::read_local(&pool)
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
        scheduled_tasks::setup(db_url, user_agent, context).expect("Couldn't set up scheduled_tasks");
      }
    });
  }

  // Create Http server with websocket support
  let settings_bind = settings.clone();
  HttpServer::new(move || {
    let context = context.clone();
    let federation_config = FederationConfig::builder()
      .domain(settings.hostname.clone())
      .app_data(context.clone())
      .client(client.clone())
      .http_fetch_limit(FEDERATION_HTTP_FETCH_LIMIT)
      .worker_count(local_site.federation_worker_count as u64)
      .debug(cfg!(debug_assertions))
      .http_signature_compat(true)
      .url_verifier(Box::new(VerifyUrlData(context.pool().clone())))
      .build()
      .expect("configure federation");

    let cors_config = if cfg!(debug_assertions) {
      Cors::permissive()
    } else {
      Cors::default()
    };

    App::new()
      .wrap(middleware::Logger::default())
      .wrap(cors_config)
      .wrap(TracingLogger::<QuieterRootSpanBuilder>::new())
      .app_data(Data::new(context))
      .app_data(Data::new(rate_limit_cell.clone()))
      .wrap(FederationMiddleware::new(federation_config))
      // The routes
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

  Ok(())
}

pub fn init_logging(opentelemetry_url: &Option<Url>) -> Result<(), LemmyError> {
  LogTracer::init()?;

  let log_description = std::env::var("RUST_LOG").unwrap_or_else(|_| "info".into());

  let targets = log_description
    .trim()
    .trim_matches('"')
    .parse::<Targets>()?;

  let format_layer = tracing_subscriber::fmt::layer().with_filter(targets.clone());

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
