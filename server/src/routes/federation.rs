use super::*;
use crate::apub::community::*;
use crate::apub::community_inbox::community_inbox;
use crate::apub::post::get_apub_post;
use crate::apub::shared_inbox::shared_inbox;
use crate::apub::user::*;
use crate::apub::user_inbox::user_inbox;
use crate::apub::APUB_JSON_CONTENT_TYPE;

pub fn config(cfg: &mut web::ServiceConfig) {
  if Settings::get().federation.enabled {
    println!("federation enabled, host is {}", Settings::get().hostname);
    cfg
      .service(
        web::scope("/")
          .guard(guard::Header("Accept", APUB_JSON_CONTENT_TYPE))
          .route(
            "/c/{community_name}",
            web::get().to(get_apub_community_http),
          )
          .route(
            "/c/{community_name}/followers",
            web::get().to(get_apub_community_followers),
          )
          // TODO This is only useful for history which we aren't doing right now
          // .route(
          //   "/c/{community_name}/outbox",
          //   web::get().to(get_apub_community_outbox),
          // )
          .route("/u/{user_name}", web::get().to(get_apub_user_http))
          .route("/post/{post_id}", web::get().to(get_apub_post)),
      )
      // Inboxes dont work with the header guard for some reason.
      .route("/c/{community_name}/inbox", web::post().to(community_inbox))
      .route("/u/{user_name}/inbox", web::post().to(user_inbox))
      .route("/inbox", web::post().to(shared_inbox));
  }
}
