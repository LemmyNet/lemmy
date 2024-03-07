use activitypub_federation::{
  config::Data,
  fetch::webfinger::{extract_webfinger_name, Webfinger, WebfingerLink, WEBFINGER_CONTENT_TYPE},
};
use actix_web::{web, web::Query, HttpResponse};
use lemmy_api_common::context::LemmyContext;
use lemmy_db_schema::{
  source::{community::Community, person::Person},
  traits::ApubActor,
  CommunityVisibility,
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

  let user_id: Option<Url> = Person::read_from_name(&mut context.pool(), name, false)
    .await
    .ok()
    .map(|c| c.actor_id.into());
  let community_id: Option<Url> = Community::read_from_name(&mut context.pool(), name, false)
    .await
    .ok()
    .and_then(|c| {
      if c.visibility == CommunityVisibility::Public {
        let id: Url = c.actor_id.into();
        Some(id)
      } else {
        None
      }
    });

  // Mastodon seems to prioritize the last webfinger item in case of duplicates. Put
  // community last so that it gets prioritized. For Lemmy the order doesnt matter.
  let links = vec![
    webfinger_link_for_actor(user_id, "Person", &context),
    webfinger_link_for_actor(community_id, "Group", &context),
  ]
  .into_iter()
  .flatten()
  .collect();

  let json = Webfinger {
    subject: info.resource.clone(),
    links,
    ..Default::default()
  };

  Ok(
    HttpResponse::Ok()
      .content_type(&WEBFINGER_CONTENT_TYPE)
      .json(json),
  )
}

fn webfinger_link_for_actor(
  url: Option<Url>,
  kind: &str,
  context: &LemmyContext,
) -> Vec<WebfingerLink> {
  if let Some(url) = url {
    let type_key = "https://www.w3.org/ns/activitystreams#type"
      .parse()
      .expect("parse url");

    let mut vec = vec![
      WebfingerLink {
        rel: Some("http://webfinger.net/rel/profile-page".into()),
        kind: Some("text/html".into()),
        href: Some(url.clone()),
        ..Default::default()
      },
      WebfingerLink {
        rel: Some("self".into()),
        kind: Some("application/activity+json".into()),
        href: Some(url),
        properties: HashMap::from([(type_key, kind.into())]),
        ..Default::default()
      },
    ];

    // insert remote follow link
    if kind == "Person" {
      let template = format!(
        "{}/activitypub/externalInteraction?uri={{uri}}",
        context.settings().get_protocol_and_hostname()
      );
      vec.push(WebfingerLink {
        rel: Some("http://ostatus.org/schema/1.0/subscribe".into()),
        template: Some(template),
        ..Default::default()
      });
    }
    vec
  } else {
    vec![]
  }
}
