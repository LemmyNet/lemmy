use crate::apub::{
  comment::get_apub_comment,
  community::*,
  inbox::{community_inbox::community_inbox, shared_inbox::shared_inbox, user_inbox::user_inbox},
  post::get_apub_post,
  user::*,
  APUB_JSON_CONTENT_TYPE,
};
use actix_web::*;
use http_signature_normalization_actix::digest::middleware::VerifyDigest;
use lemmy_utils::settings::Settings;
use sha2::{Digest, Sha256};

pub fn config(cfg: &mut web::ServiceConfig) {
  if Settings::get().federation.enabled {
    println!("federation enabled, host is {}", Settings::get().hostname);
    let digest_verifier = VerifyDigest::new(Sha256::new());

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
          .route(
            "/c/{community_name}/outbox",
            web::get().to(get_apub_community_outbox),
          )
          .route("/u/{user_name}", web::get().to(get_apub_user_http))
          .route("/post/{post_id}", web::get().to(get_apub_post))
          .route("/comment/{comment_id}", web::get().to(get_apub_comment)),
      )
      // Inboxes dont work with the header guard for some reason.
      .service(
        web::resource("/c/{community_name}/inbox")
          .wrap(digest_verifier.clone())
          .route(web::post().to(community_inbox)),
      )
      .service(
        web::resource("/u/{user_name}/inbox")
          .wrap(digest_verifier.clone())
          .route(web::post().to(user_inbox)),
      )
      .service(
        web::resource("/inbox")
          .wrap(digest_verifier)
          .route(web::post().to(shared_inbox)),
      );
  }
}
