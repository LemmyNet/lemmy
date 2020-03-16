pub mod community;
pub mod post;
pub mod puller;
pub mod user;
use crate::Settings;

use actix_web::body::Body;
use actix_web::HttpResponse;
use std::fmt::Display;
use url::Url;

fn create_apub_response(json_data: String) -> HttpResponse<Body> {
  HttpResponse::Ok()
    .content_type("application/activity+json")
    .body(json_data)
}

// TODO: this should take an enum community/user/post for `point`
// TODO: also not sure what exactly `value` should be (numeric id, name string, ...)
fn make_apub_endpoint<S: Display, T: Display>(point: S, value: T) -> Url {
  Url::parse(&format!(
    "{}://{}/federation/{}/{}",
    get_apub_protocol_string(),
    Settings::get().hostname,
    point,
    value
  ))
  .unwrap()
}

fn get_apub_protocol_string() -> &'static str {
  "http"
}
