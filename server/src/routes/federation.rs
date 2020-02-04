use crate::apub;
use actix_web::web;

pub fn config(cfg: &mut web::ServiceConfig) {
  cfg
    .route(
      "/federation/c/{community_name}",
      web::get().to(apub::community::get_apub_community),
    )
    .route(
      "/federation/c/{community_name}/followers",
      web::get().to(apub::community::get_apub_community_followers),
    )
    .route(
      "/federation/u/{user_name}",
      web::get().to(apub::user::get_apub_user),
    );
}
