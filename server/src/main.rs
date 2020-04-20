extern crate lemmy_server;
#[macro_use]
extern crate diesel_migrations;

use actix::prelude::*;
use actix_web::*;
use diesel::r2d2::{ConnectionManager, Pool};
use diesel::PgConnection;
use lemmy_server::{
  rate_limit::{rate_limiter::RateLimiter, RateLimit},
  routes::{api, federation, feeds, index, nodeinfo, webfinger},
  settings::Settings,
  websocket::server::*,
};
use std::{io, sync::Arc};
use tokio::sync::Mutex;

embed_migrations!();

#[actix_rt::main]
async fn main() -> io::Result<()> {
  env_logger::init();
  let settings = Settings::get();

  // Set up the r2d2 connection pool
  let manager = ConnectionManager::<PgConnection>::new(&settings.get_database_url());
  let pool = Pool::builder()
    .max_size(settings.database.pool_size)
    .build(manager)
    .unwrap_or_else(|_| panic!("Error connecting to {}", settings.get_database_url()));

  // Run the migrations from code
  let conn = pool.get().unwrap();
  embedded_migrations::run(&conn).unwrap();

  // Set up the rate limiter
  let rate_limiter = RateLimit {
    rate_limiter: Arc::new(Mutex::new(RateLimiter::default())),
  };

  // Set up websocket server
  let server = ChatServer::startup(pool.clone(), rate_limiter.clone()).start();

  println!(
    "Starting http server at {}:{}",
    settings.bind, settings.port
  );

  // Create Http server with websocket support
  HttpServer::new(move || {
    let settings = Settings::get();
    let rate_limiter = rate_limiter.clone();
    App::new()
      .wrap(middleware::Logger::default())
      .data(pool.clone())
      .data(server.clone())
      // The routes
      .configure(move |cfg| api::config(cfg, &rate_limiter))
      .configure(federation::config)
      .configure(feeds::config)
      .configure(index::config)
      .configure(nodeinfo::config)
      .configure(webfinger::config)
      // static files
      .service(actix_files::Files::new(
        "/static",
        settings.front_end_dir.to_owned(),
      ))
      .service(actix_files::Files::new(
        "/docs",
        settings.front_end_dir + "/documentation",
      ))
  })
  .bind((settings.bind, settings.port))?
  .run()
  .await
}
