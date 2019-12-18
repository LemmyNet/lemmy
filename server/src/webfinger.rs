use crate::db::community::Community;
use crate::db::establish_connection;
use crate::Settings;
use actix_web::body::Body;
use actix_web::web::Query;
use actix_web::HttpResponse;
use serde::Deserialize;
use serde_json::json;

#[derive(Deserialize)]
pub struct Params {
  resource: String,
}

/// Responds to webfinger requests of the following format. There isn't any real documentation for
/// this, but it described in this blog post:
/// https://mastodon.social/.well-known/webfinger?resource=acct:gargron@mastodon.social
///
/// You can also view the webfinger response that Mastodon sends:
/// https://radical.town/.well-known/webfinger?resource=acct:felix@radical.town
pub fn get_webfinger_response(info: Query<Params>) -> HttpResponse<Body> {
  // NOTE: Calling the parameter "account" maybe doesn't really make sense, but should give us the
  // best compatibility with existing implementations. We could also support an alternative name
  // like "group", and encourage others to use that.
  let community_identifier = info.resource.replace("acct:", "");
  let split_identifier: Vec<&str> = community_identifier.split("@").collect();
  let community_name = split_identifier[0];
  // It looks like Mastodon does not return webfinger requests for users from other instances, so we
  // don't do that either.
  if split_identifier.len() != 2 || split_identifier[1] != Settings::get().hostname {
    return HttpResponse::NotFound().finish();
  }

  // Make sure the requested community exists.
  let conn = establish_connection();
  match Community::read_from_name(&conn, community_name.to_owned()) {
    Err(_) => return HttpResponse::NotFound().finish(),
    Ok(c) => c,
  };

  let community_url = Community::get_community_url(&community_name);

  let json = json!({
    "subject": info.resource,
    "aliases": [
      community_url,
    ],
    "links": [
      {
        "rel": "http://webfinger.net/rel/profile-page",
        "type": "text/html",
        "href": community_url
      },
      {
        "rel": "self",
        "type": "application/activity+json",
        "href": community_url // Yes this is correct, this link doesn't include the `.json` extension
      }
      // TODO: this also needs to return the subscribe link once that's implemented
      //{
      //  "rel": "http://ostatus.org/schema/1.0/subscribe",
      //  "template": "https://my_instance.com/authorize_interaction?uri={uri}"
      //}
    ]
  });
  return HttpResponse::Ok()
    .content_type("application/activity+json")
    .body(json.to_string());
}
