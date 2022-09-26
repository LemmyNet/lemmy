#[macro_use]
extern crate diesel_migrations;

use crate::diesel_migrations::MigrationHarness;
use actix::prelude::*;
use actix_web::{web::Data, *};
use diesel::{
  r2d2::{ConnectionManager, Pool},
  PgConnection,
};
use diesel_migrations::EmbeddedMigrations;
use doku::json::{AutoComments, Formatting};
use lemmy_api::match_websocket_operation;
use lemmy_api_common::{
  request::build_user_agent,
  utils::{blocking, check_private_instance_and_federation_enabled},
};
use lemmy_api_crud::match_websocket_operation_crud;
use lemmy_db_schema::{source::secret::Secret, utils::get_database_url_from_env};
use lemmy_routes::{feeds, images, nodeinfo, webfinger};
use lemmy_server::{
  api_routes,
  code_migrations::run_advanced_migrations,
  init_logging,
  root_span_builder::QuieterRootSpanBuilder,
  scheduled_tasks,
};
use lemmy_utils::{
  error::LemmyError,
  rate_limit::{rate_limiter::RateLimiter, RateLimit},
  settings::{structs::Settings, SETTINGS},
};
use lemmy_websocket::{chat_server::ChatServer, LemmyContext};
use reqwest::Client;
use reqwest_middleware::ClientBuilder;
use reqwest_retry::{policies::ExponentialBackoff, RetryTransientMiddleware};
use reqwest_tracing::TracingMiddleware;
use std::{
  env,
  sync::{Arc, Mutex},
  thread,
  time::Duration,
};
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
      ..Default::default()
    };
    println!("{}", doku::to_json_fmt_val(&fmt, &Settings::default()));
    return Ok(());
  }

  let settings = SETTINGS.to_owned();

  init_logging(&settings.opentelemetry_url)?;

  // Set up the r2d2 connection pool
  let db_url = match get_database_url_from_env() {
    Ok(url) => url,
    Err(_) => settings.get_database_url(),
  };
  let manager = ConnectionManager::<PgConnection>::new(&db_url);
  let pool = Pool::builder()
    .max_size(settings.database.pool_size)
    .min_idle(Some(1))
    .build(manager)
    .unwrap_or_else(|_| panic!("Error connecting to {}", db_url));

  // Run the migrations from code
  let protocol_and_hostname = settings.get_protocol_and_hostname();
  blocking(&pool, move |conn| {
    let _ = conn
      .run_pending_migrations(MIGRATIONS)
      .map_err(|_| LemmyError::from_message("Couldn't run migrations"))?;
    run_advanced_migrations(conn, &protocol_and_hostname)?;
    Ok(()) as Result<(), LemmyError>
  })
  .await??;

  // Schedules various cleanup tasks for the DB
  let pool2 = pool.clone();
  thread::spawn(move || {
    scheduled_tasks::setup(pool2).expect("Couldn't set up scheduled_tasks");
  });

  // Set up the rate limiter
  let rate_limiter = RateLimit {
    rate_limiter: Arc::new(Mutex::new(RateLimiter::default())),
    rate_limit_config: settings.rate_limit.to_owned().unwrap_or_default(),
  };

  // Initialize the secrets
  let conn = &mut pool.get()?;
  let secret = Secret::init(conn).expect("Couldn't initialize secrets.");

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

  check_private_instance_and_federation_enabled(&pool, &settings).await?;

  let chat_server = ChatServer::startup(
    pool.clone(),
    rate_limiter.clone(),
    |c, i, o, d| Box::pin(match_websocket_operation(c, i, o, d)),
    |c, i, o, d| Box::pin(match_websocket_operation_crud(c, i, o, d)),
    client.clone(),
    settings.clone(),
    secret.clone(),
  )
  .start();

  // Create Http server with websocket support
  let settings_bind = settings.clone();
  HttpServer::new(move || {
    let context = LemmyContext::create(
      pool.clone(),
      chat_server.to_owned(),
      client.clone(),
      settings.to_owned(),
      secret.to_owned(),
    );
    let rate_limiter = rate_limiter.clone();
    App::new()
      .wrap(actix_web::middleware::Logger::default())
      .wrap(TracingLogger::<QuieterRootSpanBuilder>::new())
      .app_data(Data::new(context))
      .app_data(Data::new(rate_limiter.clone()))
      // The routes
      .configure(|cfg| api_routes::config(cfg, &rate_limiter))
      .configure(|cfg| lemmy_apub::http::routes::config(cfg, &settings))
      .configure(feeds::config)
      .configure(|cfg| images::config(cfg, pictrs_client.clone(), &rate_limiter))
      .configure(nodeinfo::config)
      .configure(|cfg| webfinger::config(cfg, &settings))
  })
  .bind((settings_bind.bind, settings_bind.port))?
  .run()
  .await?;

  Ok(())
}
