use crate::{
  blocking,
  db::{community::Community, user::User_},
  routes::DbPoolParam,
  LemmyError,
  Settings,
};
use actix_web::{error::ErrorBadRequest, web::Query, *};
use regex::Regex;
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct Params {
  resource: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct WebFingerResponse {
  pub subject: String,
  pub aliases: Vec<String>,
  pub links: Vec<WebFingerLink>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct WebFingerLink {
  pub rel: Option<String>,
  #[serde(rename(serialize = "type", deserialize = "type"))]
  pub type_: Option<String>,
  pub href: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub template: Option<String>,
}

pub fn config(cfg: &mut web::ServiceConfig) {
  if Settings::get().federation.enabled {
    cfg.route(
      ".well-known/webfinger",
      web::get().to(get_webfinger_response),
    );
  }
}

lazy_static! {
  static ref WEBFINGER_COMMUNITY_REGEX: Regex = Regex::new(&format!(
    "^group:([a-z0-9_]{{3, 20}})@{}$",
    Settings::get().hostname
  ))
  .unwrap();
  static ref WEBFINGER_USER_REGEX: Regex = Regex::new(&format!(
    "^acct:([a-z0-9_]{{3, 20}})@{}$",
    Settings::get().hostname
  ))
  .unwrap();
}

/// Responds to webfinger requests of the following format. There isn't any real documentation for
/// this, but it described in this blog post:
/// https://mastodon.social/.well-known/webfinger?resource=acct:gargron@mastodon.social
///
/// You can also view the webfinger response that Mastodon sends:
/// https://radical.town/.well-known/webfinger?resource=acct:felix@radical.town
async fn get_webfinger_response(
  info: Query<Params>,
  db: DbPoolParam,
) -> Result<HttpResponse, Error> {
  let community_regex_parsed = WEBFINGER_COMMUNITY_REGEX
    .captures(&info.resource)
    .map(|c| c.get(1))
    .flatten();

  let user_regex_parsed = WEBFINGER_USER_REGEX
    .captures(&info.resource)
    .map(|c| c.get(1))
    .flatten();

  let url = if let Some(community_name) = community_regex_parsed {
    let community_name = community_name.as_str().to_owned();
    // Make sure the requested community exists.
    blocking(&db, move |conn| {
      Community::read_from_name(conn, &community_name)
    })
    .await?
    .map_err(|_| ErrorBadRequest(LemmyError::from(format_err!("not_found"))))?
    .actor_id
  } else if let Some(user_name) = user_regex_parsed {
    let user_name = user_name.as_str().to_owned();
    // Make sure the requested user exists.
    blocking(&db, move |conn| User_::read_from_name(conn, &user_name))
      .await?
      .map_err(|_| ErrorBadRequest(LemmyError::from(format_err!("not_found"))))?
      .actor_id
  } else {
    return Err(ErrorBadRequest(LemmyError::from(format_err!("not_found"))));
  };

  let json = WebFingerResponse {
    subject: info.resource.to_owned(),
    aliases: vec![url.to_owned()],
    links: vec![
      WebFingerLink {
        rel: Some("http://webfinger.net/rel/profile-page".to_string()),
        type_: Some("text/html".to_string()),
        href: Some(url.to_owned()),
        template: None,
      },
      WebFingerLink {
        rel: Some("self".to_string()),
        type_: Some("application/activity+json".to_string()),
        href: Some(url),
        template: None,
      }, // TODO: this also needs to return the subscribe link once that's implemented
         //{
         //  "rel": "http://ostatus.org/schema/1.0/subscribe",
         //  "template": "https://my_instance.com/authorize_interaction?uri={uri}"
         //}
    ],
  };

  Ok(HttpResponse::Ok().json(json))
}
