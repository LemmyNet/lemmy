use super::*;

pub fn config(cfg: &mut web::ServiceConfig) {
  cfg
    .route("/", web::get().to(index))
    .route(
      "/home/data_type/{data_type}/listing_type/{listing_type}/sort/{sort}/page/{page}",
      web::get().to(index),
    )
    .route("/login", web::get().to(index))
    .route("/create_post", web::get().to(index))
    .route("/create_community", web::get().to(index))
    .route("/create_private_message", web::get().to(index))
    .route("/communities/page/{page}", web::get().to(index))
    .route("/communities", web::get().to(index))
    .route("/post/{id}/comment/{id2}", web::get().to(index))
    .route("/post/{id}", web::get().to(index))
    .route(
      "/c/{name}/data_type/{data_type}/sort/{sort}/page/{page}",
      web::get().to(index),
    )
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
    .route("/admin", web::get().to(index))
    .route(
      "/search/q/{q}/type/{type}/sort/{sort}/page/{page}",
      web::get().to(index),
    )
    .route("/search", web::get().to(index))
    .route("/sponsors", web::get().to(index))
    .route("/password_change/{token}", web::get().to(index));
}

async fn index() -> Result<NamedFile, Error> {
  Ok(NamedFile::open(
    Settings::get().front_end_dir + "/index.html",
  )?)
}
