extern crate lemmy_server;
#[macro_use]
extern crate diesel_migrations;
#[macro_use]
pub extern crate lazy_static;

use crate::lemmy_server::actix_web::dev::Service;
use actix::prelude::*;
use actix_web::{
  body::Body,
  dev::{ServiceRequest, ServiceResponse},
  http::{
    header::{CACHE_CONTROL, CONTENT_TYPE},
    HeaderValue,
  },
  *,
};
use diesel::{
  r2d2::{ConnectionManager, Pool},
  PgConnection,
};
use lemmy_server::{
  db::code_migrations::run_advanced_migrations,
  rate_limit::{rate_limiter::RateLimiter, RateLimit},
  routes::{api, federation, feeds, index, nodeinfo, webfinger},
  settings::Settings,
  websocket::server::*,
};
use regex::Regex;
use std::{io, sync::Arc};
use tokio::sync::Mutex;

lazy_static! {
  static ref CACHE_CONTROL_REGEX: Regex =
    Regex::new("^((text|image)/.+|application/javascript)$").unwrap();
  // static ref CACHE_CONTROL_VALUE: String = format!("public, max-age={}", 365 * 24 * 60 * 60);
  // Test out 1 hour here, this is breaking some things
  static ref CACHE_CONTROL_VALUE: String = format!("public, max-age={}", 60 * 60);
}

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
  run_advanced_migrations(&conn).unwrap();

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
      .wrap_fn(add_cache_headers)
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

fn add_cache_headers<S>(
  req: ServiceRequest,
  srv: &mut S,
) -> impl Future<Output = Result<ServiceResponse, Error>>
where
  S: Service<Request = ServiceRequest, Response = ServiceResponse<Body>, Error = Error>,
{
  let fut = srv.call(req);
  async move {
    let mut res = fut.await?;
    if let Some(content_type) = res.headers().get(CONTENT_TYPE) {
      if CACHE_CONTROL_REGEX.is_match(content_type.to_str().unwrap()) {
        let header_val = HeaderValue::from_static(&CACHE_CONTROL_VALUE);
        res.headers_mut().insert(CACHE_CONTROL, header_val);
      }
    }
    Ok(res)
  }
}
