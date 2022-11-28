#[macro_use]
extern crate diesel_migrations;

use actix::prelude::*;
use actix_web::{middleware, web::Data, App, HttpServer, Result};
use diesel_migrations::EmbeddedMigrations;
use doku::json::{AutoComments, CommentsStyle, Formatting, ObjectsStyle};
use lemmy_api_common::{
  context::LemmyContext,
  lemmy_db_views::structs::SiteView,
  request::build_user_agent,
  utils::{
    check_private_instance_and_federation_enabled,
    local_site_rate_limit_to_rate_limit_config,
  },
  websocket::chat_server::ChatServer,
};
use lemmy_db_schema::{
  source::secret::Secret,
  utils::{build_db_pool, get_database_url, run_migrations},
};
use lemmy_routes::{feeds, images, nodeinfo, webfinger};
use lemmy_server::{
  api_routes,
  api_routes::{
    match_websocket_operation,
    match_websocket_operation_apub,
    match_websocket_operation_crud,
  },
  code_migrations::run_advanced_migrations,
  init_logging,
  root_span_builder::QuieterRootSpanBuilder,
  scheduled_tasks,
};
use lemmy_utils::{
  error::LemmyError,
  rate_limit::RateLimitCell,
  settings::{structs::Settings, SETTINGS},
};
use reqwest::Client;
use reqwest_middleware::ClientBuilder;
use reqwest_retry::{policies::ExponentialBackoff, RetryTransientMiddleware};
use reqwest_tracing::TracingMiddleware;
use std::{env, thread, time::Duration};
use tracing_actix_web::TracingLogger;

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!();

/// Max timeout for http requests
pub const REQWEST_TIMEOUT: Duration = Duration::from_secs(10);

#[actix_web::main]
async fn main() -> Result<(), LemmyError> {
  let args: Vec<String> = env::args().collect();
  if args.len() == 2 && args[1] == "--print-config-docs" {
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

  let settings = SETTINGS.to_owned();

  init_logging(&settings.opentelemetry_url)?;

  // Set up the bb8 connection pool
  let db_url = get_database_url(Some(&settings));
  run_migrations(&db_url);

  // Run the migrations from code
  let pool = build_db_pool(&settings).await?;
  run_advanced_migrations(&pool, &settings).await?;

  // Schedules various cleanup tasks for the DB
  thread::spawn(move || {
    scheduled_tasks::setup(db_url).expect("Couldn't set up scheduled_tasks");
  });

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

  let reqwest_client = Client::builder()
    .user_agent(build_user_agent(&settings))
    .timeout(REQWEST_TIMEOUT)
    .build()?;

  let retry_policy = ExponentialBackoff {
    max_n_retries: 3,
    max_retry_interval: REQWEST_TIMEOUT,
    min_retry_interval: Duration::from_millis(100),
    backoff_exponent: 2,
  };

  let client = ClientBuilder::new(reqwest_client.clone())
    .with(TracingMiddleware::default())
    .with(RetryTransientMiddleware::new_with_policy(retry_policy))
    .build();

  // Pictrs cannot use the retry middleware
  let pictrs_client = ClientBuilder::new(reqwest_client.clone())
    .with(TracingMiddleware::default())
    .build();

  let chat_server = ChatServer::startup(
    pool.clone(),
    |c, i, o, d| Box::pin(match_websocket_operation(c, i, o, d)),
    |c, i, o, d| Box::pin(match_websocket_operation_crud(c, i, o, d)),
    |c, i, o, d| Box::pin(match_websocket_operation_apub(c, i, o, d)),
    client.clone(),
    settings.clone(),
    secret.clone(),
    rate_limit_cell.clone(),
  )
  .start();

  // Create Http server with websocket support
  let settings_bind = settings.clone();
  HttpServer::new(move || {
    let context = LemmyContext::create(
      pool.clone(),
      chat_server.clone(),
      client.clone(),
      settings.clone(),
      secret.clone(),
      rate_limit_cell.clone(),
    );
    App::new()
      .wrap(middleware::Logger::default())
      .wrap(TracingLogger::<QuieterRootSpanBuilder>::new())
      .app_data(Data::new(context))
      .app_data(Data::new(rate_limit_cell.clone()))
      // The routes
      .configure(|cfg| api_routes::config(cfg, rate_limit_cell))
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
