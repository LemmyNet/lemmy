use crate::apub;
use crate::settings::Settings;
use actix_web::web;

pub fn config(cfg: &mut web::ServiceConfig) {
  if Settings::get().federation.enabled {
    println!("federation enabled, host is {}", Settings::get().hostname);
    cfg
      // TODO: check the user/community params for these
      .route(
        "/federation/c/{_}/inbox",
        web::post().to(apub::community_inbox::community_inbox),
      )
      .route(
        "/federation/u/{_}/inbox",
        web::post().to(apub::user_inbox::user_inbox),
      )
      .route(
        "/federation/c/{community_name}",
        web::get().to(apub::community::get_apub_community_http),
      )
      .route(
        "/federation/c/{community_name}/followers",
        web::get().to(apub::community::get_apub_community_followers),
      )
      .route(
        "/federation/c/{community_name}/outbox",
        web::get().to(apub::community::get_apub_community_outbox),
      )
      .route(
        "/federation/u/{user_name}",
        web::get().to(apub::user::get_apub_user),
      )
      .route(
        "/federation/p/{post_id}",
        web::get().to(apub::post::get_apub_post),
      );
  }
}
