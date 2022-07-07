use actix_web::{web, web::Query, HttpResponse};
use anyhow::Context;
use lemmy_api_common::utils::blocking;
use lemmy_apub::fetcher::webfinger::{WebfingerLink, WebfingerResponse};
use lemmy_db_schema::{
  source::{community::Community, person::Person},
  traits::ApubActor,
};
use lemmy_utils::{error::LemmyError, location_info, settings::structs::Settings};
use lemmy_websocket::LemmyContext;
use serde::Deserialize;
use url::Url;

#[derive(Deserialize)]
struct Params {
  resource: String,
}

pub fn config(cfg: &mut web::ServiceConfig, settings: &Settings) {
  if settings.federation.enabled {
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
) -> Result<HttpResponse, LemmyError> {
  let name = context
    .settings()
    .webfinger_regex()
    .captures(&info.resource)
    .and_then(|c| c.get(1))
    .context(location_info!())?
    .as_str()
    .to_string();

  let name_ = name.clone();
  let user_id: Option<Url> = blocking(context.pool(), move |conn| {
    Person::read_from_name(conn, &name_, false)
  })
  .await?
  .ok()
  .map(|c| c.actor_id.into());
  let community_id: Option<Url> = blocking(context.pool(), move |conn| {
    Community::read_from_name(conn, &name, false)
  })
  .await?
  .ok()
  .map(|c| c.actor_id.into());

  // Mastodon seems to prioritize the last webfinger item in case of duplicates. Put
  // community last so that it gets prioritized. For Lemmy the order doesnt matter.
  let links = vec![
    webfinger_link_for_actor(user_id),
    webfinger_link_for_actor(community_id),
  ]
  .into_iter()
  .flatten()
  .collect();

  let json = WebfingerResponse {
    subject: info.resource.to_owned(),
    links,
  };

  Ok(HttpResponse::Ok().json(json))
}

fn webfinger_link_for_actor(url: Option<Url>) -> Vec<WebfingerLink> {
  if let Some(url) = url {
    vec![
      WebfingerLink {
        rel: Some("http://webfinger.net/rel/profile-page".to_string()),
        kind: Some("text/html".to_string()),
        href: Some(url.to_owned()),
      },
      WebfingerLink {
        rel: Some("self".to_string()),
        kind: Some("application/activity+json".to_string()),
        href: Some(url),
      },
    ]
  } else {
    vec![]
  }
}
