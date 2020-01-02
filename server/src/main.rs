extern crate lemmy_server;
#[macro_use]
extern crate diesel_migrations;

use actix_web::*;
use lemmy_server::db::establish_connection;
use lemmy_server::routes::{federation, feeds, index, nodeinfo, webfinger, websocket};
use lemmy_server::settings::Settings;

embed_migrations!();

fn main() {
  env_logger::init();
  let sys = actix::System::new("lemmy");

  // Run the migrations from code
  let conn = establish_connection();
  embedded_migrations::run(&conn).unwrap();

  let settings = Settings::get();

  // Create Http server with websocket support
  HttpServer::new(move || {
    App::new()
      .configure(federation::config)
      .configure(feeds::config)
      .configure(index::config)
      .configure(nodeinfo::config)
      .configure(webfinger::config)
      .configure(websocket::config)
      .service(actix_files::Files::new(
        "/static",
        settings.front_end_dir.to_owned(),
      ))
      .service(actix_files::Files::new(
        "/docs",
        settings.front_end_dir.to_owned() + "/documentation",
      ))
  })
  .bind((settings.bind, settings.port))
  .unwrap()
  .start();

  println!("Started http server at {}:{}", settings.bind, settings.port);
  let _ = sys.run();
}
