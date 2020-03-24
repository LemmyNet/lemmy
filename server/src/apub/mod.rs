pub mod community;
pub mod post;
pub mod puller;
pub mod user;
use crate::Settings;

use actix_web::body::Body;
use actix_web::HttpResponse;
use url::Url;

fn create_apub_response<T>(json: &T) -> HttpResponse<Body>
where
  T: serde::ser::Serialize,
{
  HttpResponse::Ok()
    .content_type("application/activity+json")
    .json(json)
}

enum EndpointType {
  Community,
  User,
  Post,
}

fn make_apub_endpoint(endpoint_type: EndpointType, name: &str) -> Url {
  let point = match endpoint_type {
    EndpointType::Community => "c",
    EndpointType::User => "u",
    EndpointType::Post => "p",
  };

  Url::parse(&format!(
    "{}://{}/federation/{}/{}",
    get_apub_protocol_string(),
    Settings::get().hostname,
    point,
    name
  ))
  .unwrap()
}

pub fn get_apub_protocol_string() -> &'static str {
  if Settings::get().federation.tls_enabled {
    "https"
  } else {
    "http"
  }
}
