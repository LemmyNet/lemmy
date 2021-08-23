use actix_web::{error::ErrorBadRequest, web::Query, *};
use anyhow::anyhow;
use lemmy_api_common::blocking;
use lemmy_apub_lib::webfinger::{WebfingerLink, WebfingerResponse};
use lemmy_db_queries::source::{community::Community_, person::Person_};
use lemmy_db_schema::source::{community::Community, person::Person};
use lemmy_utils::{
  settings::structs::Settings,
  LemmyError,
  WEBFINGER_COMMUNITY_REGEX,
  WEBFINGER_USERNAME_REGEX,
};
use lemmy_websocket::LemmyContext;
use serde::Deserialize;

#[derive(Deserialize)]
struct Params {
  resource: String,
}

pub fn config(cfg: &mut web::ServiceConfig) {
  if Settings::get().federation.enabled {
    cfg.route(
      ".well-known/webfinger",
      web::get().to(get_webfinger_response),
    );
  }
}

/// Responds to webfinger requests of the following format. There isn't any real documentation for
/// this, but it described in this blog post:
/// https://mastodon.social/.well-known/webfinger?resource=acct:gargron@mastodon.social
///
/// You can also view the webfinger response that Mastodon sends:
/// https://radical.town/.well-known/webfinger?resource=acct:felix@radical.town
async fn get_webfinger_response(
  info: Query<Params>,
  context: web::Data<LemmyContext>,
) -> Result<HttpResponse, Error> {
  let community_regex_parsed = WEBFINGER_COMMUNITY_REGEX
    .captures(&info.resource)
    .map(|c| c.get(1))
    .flatten();

  let username_regex_parsed = WEBFINGER_USERNAME_REGEX
    .captures(&info.resource)
    .map(|c| c.get(1))
    .flatten();

  let url = if let Some(community_name) = community_regex_parsed {
    let community_name = community_name.as_str().to_owned();
    // Make sure the requested community exists.
    blocking(context.pool(), move |conn| {
      Community::read_from_name(conn, &community_name)
    })
    .await?
    .map_err(|_| ErrorBadRequest(LemmyError::from(anyhow!("not_found"))))?
    .actor_id
  } else if let Some(person_name) = username_regex_parsed {
    let person_name = person_name.as_str().to_owned();
    // Make sure the requested person exists.
    blocking(context.pool(), move |conn| {
      Person::find_by_name(conn, &person_name)
    })
    .await?
    .map_err(|_| ErrorBadRequest(LemmyError::from(anyhow!("not_found"))))?
    .actor_id
  } else {
    return Err(ErrorBadRequest(LemmyError::from(anyhow!("not_found"))));
  };

  let json = WebfingerResponse {
    subject: info.resource.to_owned(),
    aliases: vec![url.to_owned().into()],
    links: vec![
      WebfingerLink {
        rel: Some("http://webfinger.net/rel/profile-page".to_string()),
        type_: Some("text/html".to_string()),
        href: Some(url.to_owned().into()),
        template: None,
      },
      WebfingerLink {
        rel: Some("self".to_string()),
        type_: Some("application/activity+json".to_string()),
        href: Some(url.into()),
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
