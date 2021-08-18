#[macro_use]
extern crate diesel_migrations;

use actix::prelude::*;
use actix_web::{web::Data, *};
use diesel::{
  r2d2::{ConnectionManager, Pool},
  PgConnection,
};
use lemmy_api::match_websocket_operation;
use lemmy_api_common::blocking;
use lemmy_api_crud::match_websocket_operation_crud;
use lemmy_apub::activity_queue::create_activity_queue;
use lemmy_db_queries::get_database_url_from_env;
use lemmy_routes::{feeds, images, nodeinfo, webfinger};
use lemmy_server::{api_routes, code_migrations::run_advanced_migrations, scheduled_tasks};
use lemmy_utils::{
  rate_limit::{rate_limiter::RateLimiter, RateLimit},
  settings::structs::Settings,
  LemmyError,
};
use lemmy_websocket::{chat_server::ChatServer, LemmyContext};
use reqwest::Client;
use std::{sync::Arc, thread};
use tokio::sync::Mutex;

embed_migrations!();

#[actix_web::main]
async fn main() -> Result<(), LemmyError> {
  env_logger::init();
  let settings = Settings::get();

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
  blocking(&pool, move |conn| {
    embedded_migrations::run(conn)?;
    run_advanced_migrations(conn)?;
    Ok(()) as Result<(), LemmyError>
  })
  .await??;

  let pool2 = pool.clone();
  thread::spawn(move || {
    scheduled_tasks::setup(pool2);
  });

  // Set up the rate limiter
  let rate_limiter = RateLimit {
    rate_limiter: Arc::new(Mutex::new(RateLimiter::default())),
  };

  println!(
    "Starting http server at {}:{}",
    settings.bind, settings.port
  );

  let activity_queue = create_activity_queue();

  let chat_server = ChatServer::startup(
    pool.clone(),
    rate_limiter.clone(),
    |c, i, o, d| Box::pin(match_websocket_operation(c, i, o, d)),
    |c, i, o, d| Box::pin(match_websocket_operation_crud(c, i, o, d)),
    Client::default(),
    activity_queue.clone(),
  )
  .start();

  // Create Http server with websocket support
  HttpServer::new(move || {
    let context = LemmyContext::create(
      pool.clone(),
      chat_server.to_owned(),
      Client::default(),
      activity_queue.to_owned(),
    );
    let rate_limiter = rate_limiter.clone();
    App::new()
      .wrap(middleware::Logger::default())
      .app_data(Data::new(context))
      // The routes
      .configure(|cfg| api_routes::config(cfg, &rate_limiter))
      .configure(lemmy_apub::http::routes::config)
      .configure(feeds::config)
      .configure(|cfg| images::config(cfg, &rate_limiter))
      .configure(nodeinfo::config)
      .configure(webfinger::config)
  })
  .bind((settings.bind, settings.port))?
  .run()
  .await?;

  Ok(())
}
