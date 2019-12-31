use crate::settings::Settings;
use actix_files::NamedFile;
use actix_web::web;

pub fn config(cfg: &mut web::ServiceConfig) {
  cfg
    .route("/", web::get().to(index))
    .route(
      "/home/type/{type}/sort/{sort}/page/{page}",
      web::get().to(index),
    )
    .route("/login", web::get().to(index))
    .route("/create_post", web::get().to(index))
    .route("/create_community", web::get().to(index))
    .route("/communities/page/{page}", web::get().to(index))
    .route("/communities", web::get().to(index))
    .route("/post/{id}/comment/{id2}", web::get().to(index))
    .route("/post/{id}", web::get().to(index))
    .route("/c/{name}/sort/{sort}/page/{page}", web::get().to(index))
    .route("/c/{name}", web::get().to(index))
    .route("/community/{id}", web::get().to(index))
    .route(
      "/u/{username}/view/{view}/sort/{sort}/page/{page}",
      web::get().to(index),
    )
    .route("/u/{username}", web::get().to(index))
    .route("/user/{id}", web::get().to(index))
    .route("/inbox", web::get().to(index))
    .route("/modlog/community/{community_id}", web::get().to(index))
    .route("/modlog", web::get().to(index))
    .route("/setup", web::get().to(index))
    .route(
      "/search/q/{q}/type/{type}/sort/{sort}/page/{page}",
      web::get().to(index),
    )
    .route("/search", web::get().to(index))
    .route("/sponsors", web::get().to(index))
    .route("/password_change/{token}", web::get().to(index));
}

fn index() -> Result<NamedFile, actix_web::error::Error> {
  Ok(NamedFile::open(
    Settings::get().front_end_dir.to_owned() + "/index.html",
  )?)
}
