extern crate lemmy_server;
#[macro_use]
extern crate diesel_migrations;

use actix::prelude::*;
use actix_web::*;
use diesel::r2d2::{ConnectionManager, Pool};
use diesel::PgConnection;
use failure::Error;
use lemmy_server::apub::fetcher::fetch_all;
use lemmy_server::db::code_migrations::run_advanced_migrations;
use lemmy_server::routes::{api, federation, feeds, index, nodeinfo, webfinger, websocket};
use lemmy_server::settings::Settings;
use lemmy_server::websocket::server::*;
use log::warn;
use std::thread;
use std::thread::sleep;
use std::time::Duration;

embed_migrations!();

#[actix_rt::main]
async fn main() -> Result<(), Error> {
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

  // Set up websocket server
  let server = ChatServer::startup(pool.clone()).start();

  thread::spawn(move || {
    // some work here
    sleep(Duration::from_secs(5));
    println!("Fetching apub data");
    match fetch_all(&conn) {
      Ok(_) => {}
      Err(e) => warn!("Error during apub fetch: {}", e),
    }
  });

  println!(
    "Starting http server at {}:{}",
    settings.bind, settings.port
  );

  // Create Http server with websocket support
  Ok(
    HttpServer::new(move || {
      let settings = Settings::get();
      App::new()
        .wrap(middleware::Logger::default())
        .data(pool.clone())
        .data(server.clone())
        // The routes
        .configure(api::config)
        .configure(federation::config)
        .configure(feeds::config)
        .configure(index::config)
        .configure(nodeinfo::config)
        .configure(webfinger::config)
        .configure(websocket::config)
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
    .await?,
  )
}
