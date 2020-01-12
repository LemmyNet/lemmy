extern crate lemmy_server;
#[macro_use]
extern crate diesel_migrations;

use actix_web::*;
use diesel::r2d2::{ConnectionManager, Pool};
use diesel::PgConnection;
use lemmy_server::routes::{federation, feeds, index, nodeinfo, webfinger, websocket};
use lemmy_server::settings::Settings;
use std::io;

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

  println!(
    "Starting http server at {}:{}",
    settings.bind, settings.port
  );

  // Create Http server with websocket support
  HttpServer::new(move || {
    App::new()
      .wrap(middleware::Logger::default())
      .data(pool.clone())
      // The routes
      .configure(federation::config)
      .configure(feeds::config)
      .configure(index::config)
      .configure(nodeinfo::config)
      .configure(webfinger::config)
      .configure(websocket::config)
      // .configure(websocket.config(pool))
      // .configure(websocket.
      // static files
      .service(actix_files::Files::new(
        "/static",
        settings.front_end_dir.to_owned(),
      ))
      .service(actix_files::Files::new(
        "/docs",
        settings.front_end_dir.to_owned() + "/documentation",
      ))
  })
  .bind((settings.bind, settings.port))?
  .run()
  .await
}
