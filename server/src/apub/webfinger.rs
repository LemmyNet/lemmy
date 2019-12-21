use crate::apub::make_apub_endpoint;
use crate::db::community::Community;
use crate::db::establish_connection;
use crate::db::user::User_;
use crate::Settings;
use actix_web::body::Body;
use actix_web::web::Query;
use actix_web::HttpResponse;
use regex::Regex;
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct WebFingerQuery {
  resource: String,
}

#[derive(Default, Serialize, Deserialize)]
struct WebFingerResponse {
  subject: String,
  aliases: Vec<String>,
  links: Vec<WebFingerLink>,
}

#[derive(Default, Serialize, Deserialize)]
struct WebFingerLink {
  rel: String,
  r#type: String,
  href: String,
}

lazy_static! {
  static ref WEBFINGER_USER_REGEX: Regex = Regex::new(&format!(
    "^acct:@([a-zA-Z][0-9a-zA-Z_]*)@{}",
    Settings::get().hostname
  ))
  .unwrap();
  static ref WEBFINGER_COMMUNITY_REGEX: Regex = Regex::new(&format!(
    "^acct:!([a-z0-9_]{{3, 20}})@{}",
    Settings::get().hostname
  ))
  .unwrap();
}

pub fn get_webfinger(info: Query<WebFingerQuery>) -> HttpResponse<Body> {
  let connection = establish_connection();

  let (apub_page, apub_link) =
    if let Some(user_name) = WEBFINGER_USER_REGEX.captures(&info.resource) {
      let user_name = user_name.get(1).unwrap().as_str();
      if User_::find_by_email_or_username(&connection, user_name).is_err() {
        return HttpResponse::NotFound().finish();
      }

      (
        User_::get_user_url(&user_name),
        make_apub_endpoint("user", &user_name),
      )
    } else if let Some(community_name) = WEBFINGER_COMMUNITY_REGEX.captures(&info.resource) {
      let community_name = community_name.get(1).unwrap().as_str();
      if Community::read_from_name(&connection, community_name.to_owned()).is_err() {
        return HttpResponse::NotFound().finish();
      }

      (
        Community::get_community_url(&community_name),
        make_apub_endpoint("community", &community_name),
      )
    } else {
      return HttpResponse::NotFound().finish();
    };

  let response = WebFingerResponse {
    subject: info.resource.to_owned(),
    aliases: vec![apub_page.to_owned()],
    links: vec![
      WebFingerLink {
        rel: "http://webfinger.net/rel/profile-page".to_owned(),
        r#type: "text/html".to_owned(),
        href: apub_page,
      },
      WebFingerLink {
        rel: "self".to_owned(),
        r#type: "application/activity+json".to_owned(),
        href: apub_link,
      },
    ],
  };

  HttpResponse::Ok()
    .content_type("application/activity+json")
    .body(serde_json::to_string(&response).unwrap())
}
