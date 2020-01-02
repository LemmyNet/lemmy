use crate::api::community::ListCommunities;
use crate::api::Perform;
use crate::api::{Oper, UserOperation};
use crate::apub;
use crate::settings::Settings;
use actix_web::web::Query;
use actix_web::{web, HttpResponse};

pub fn config(cfg: &mut web::ServiceConfig) {
  if Settings::get().federation_enabled {
    println!("federation enabled, host is {}", Settings::get().hostname);
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
      )
      // TODO: this is a very quick and dirty implementation for http api calls
      .route(
        "/api/v1/communities/list",
        web::get().to(|query: Query<ListCommunities>| {
          let res = Oper::new(UserOperation::ListCommunities, query.into_inner())
            .perform()
            .unwrap();
          HttpResponse::Ok()
            .content_type("application/json")
            .body(serde_json::to_string(&res).unwrap())
        }),
      );
  }
}
