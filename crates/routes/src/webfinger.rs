use activitypub_federation::{
  config::Data,
  fetch::webfinger::{extract_webfinger_name, Webfinger, WebfingerLink},
};
use actix_web::{web, web::Query, HttpResponse};
use lemmy_api_common::context::LemmyContext;
use lemmy_db_schema::{
  source::{community::Community, person::Person},
  traits::ApubActor,
};
use lemmy_utils::{cache_header::cache_3days, error::LemmyError};
use serde::Deserialize;
use std::collections::HashMap;
use url::Url;

#[derive(Deserialize)]
struct Params {
  resource: String,
}

pub fn config(cfg: &mut web::ServiceConfig) {
  cfg.route(
    ".well-known/webfinger",
    web::get().to(get_webfinger_response).wrap(cache_3days()),
  );
}

/// Responds to webfinger requests of the following format. There isn't any real documentation for
/// this, but it described in this blog post:
/// https://mastodon.social/.well-known/webfinger?resource=acct:gargron@mastodon.social
///
/// You can also view the webfinger response that Mastodon sends:
/// https://radical.town/.well-known/webfinger?resource=acct:felix@radical.town
async fn get_webfinger_response(
  info: Query<Params>,
  context: Data<LemmyContext>,
) -> Result<HttpResponse, LemmyError> {
  let name = extract_webfinger_name(&info.resource, &context)?;

  let name_ = name.clone();
  let user_id: Option<Url> = Person::read_from_name(&mut context.pool(), &name_, false)
    .await
    .ok()
    .map(|c| c.actor_id.into());
  let community_id: Option<Url> = Community::read_from_name(&mut context.pool(), &name, false)
    .await
    .ok()
    .map(|c| c.actor_id.into());

  // Mastodon seems to prioritize the last webfinger item in case of duplicates. Put
  // community last so that it gets prioritized. For Lemmy the order doesnt matter.
  let links = vec![
    webfinger_link_for_actor(user_id, "Person"),
    webfinger_link_for_actor(community_id, "Group"),
  ]
  .into_iter()
  .flatten()
  .collect();

  let json = Webfinger {
    subject: info.resource.clone(),
    links,
    ..Default::default()
  };

  Ok(HttpResponse::Ok().json(json))
}

fn webfinger_link_for_actor(url: Option<Url>, kind: &str) -> Vec<WebfingerLink> {
  if let Some(url) = url {
    let mut properties = HashMap::new();
    properties.insert(
      "https://www.w3.org/ns/activitystreams#type"
        .parse()
        .expect("parse url"),
      kind.to_string(),
    );
    vec![
      WebfingerLink {
        rel: Some("http://webfinger.net/rel/profile-page".to_string()),
        kind: Some("text/html".to_string()),
        href: Some(url.clone()),
        ..Default::default()
      },
      WebfingerLink {
        rel: Some("self".to_string()),
        kind: Some("application/activity+json".to_string()),
        href: Some(url),
        properties,
      },
    ]
  } else {
    vec![]
  }
}
