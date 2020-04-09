use crate::apub;
use crate::settings::Settings;
use actix_web::web;

pub fn config(cfg: &mut web::ServiceConfig) {
  if Settings::get().federation.enabled {
    println!("federation enabled, host is {}", Settings::get().hostname);
    cfg
      .route(
        "/federation/communities",
        web::get().to(apub::community::get_apub_community_list),
      )
      .route(
        "/federation/inbox",
        web::post().to(apub::inbox::create_inbox),
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
        web::get().to(apub::user::get_apub_user),
      );
  }
}
