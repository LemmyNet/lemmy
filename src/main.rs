#[macro_use]
extern crate diesel_migrations;

use actix::prelude::*;
use actix_web::{web::Data, *};
use diesel::{
  r2d2::{ConnectionManager, Pool},
  PgConnection,
};
use doku::json::{AutoComments, Formatting};
use lemmy_api::match_websocket_operation;
use lemmy_api_common::blocking;
use lemmy_api_crud::match_websocket_operation_crud;
use lemmy_apub_lib::activity_queue::create_activity_queue;
use lemmy_db_schema::{get_database_url_from_env, source::secret::Secret};
use lemmy_routes::{feeds, images, nodeinfo, webfinger};
use lemmy_server::{
  api_routes,
  code_migrations::run_advanced_migrations,
  init_tracing,
  scheduled_tasks,
};
use lemmy_utils::{
  rate_limit::{rate_limiter::RateLimiter, RateLimit},
  request::build_user_agent,
  settings::structs::Settings,
  LemmyError,
};
use lemmy_websocket::{chat_server::ChatServer, LemmyContext};
use reqwest::Client;
use std::{env, sync::Arc, thread};
use tokio::sync::Mutex;
use tracing_actix_web::TracingLogger;

embed_migrations!();

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

  init_tracing()?;

  let settings = Settings::init().expect("Couldn't initialize settings.");

  // Set up the r2d2 connection pool
  let db_url = match get_database_url_from_env() {
    Ok(url) => url,
    Err(_) => settings.get_database_url(),
  };
  let manager = ConnectionManager::<PgConnection>::new(&db_url);
  let pool = Pool::builder()
    .max_size(settings.database.pool_size)
    .build(manager)
    .unwrap_or_else(|_| panic!("Error connecting to {}", db_url));

  // Run the migrations from code
  let protocol_and_hostname = settings.get_protocol_and_hostname();
  blocking(&pool, move |conn| {
    embedded_migrations::run(conn)?;
    run_advanced_migrations(conn, &protocol_and_hostname)?;
    Ok(()) as Result<(), LemmyError>
  })
  .await??;

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
  let conn = pool.get()?;
  let secret = Secret::init(&conn).expect("Couldn't initialize secrets.");

  println!(
    "Starting http server at {}:{}",
    settings.bind, settings.port
  );

  let client = Client::builder()
    .user_agent(build_user_agent(&settings))
    .build()?;

  let queue_manager = create_activity_queue();

  let activity_queue = queue_manager.queue_handle().clone();

  let chat_server = ChatServer::startup(
    pool.clone(),
    rate_limiter.clone(),
    |c, i, o, d| Box::pin(match_websocket_operation(c, i, o, d)),
    |c, i, o, d| Box::pin(match_websocket_operation_crud(c, i, o, d)),
    client.clone(),
    activity_queue.clone(),
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
      activity_queue.to_owned(),
      settings.to_owned(),
      secret.to_owned(),
    );
    let rate_limiter = rate_limiter.clone();
    App::new()
      .wrap(TracingLogger::default())
      .app_data(Data::new(context))
      // The routes
      .configure(|cfg| api_routes::config(cfg, &rate_limiter))
      .configure(|cfg| lemmy_apub::http::routes::config(cfg, &settings))
      .configure(feeds::config)
      .configure(|cfg| images::config(cfg, &rate_limiter))
      .configure(nodeinfo::config)
      .configure(|cfg| webfinger::config(cfg, &settings))
  })
  .bind((settings_bind.bind, settings_bind.port))?
  .run()
  .await?;

  drop(queue_manager);

  Ok(())
}
