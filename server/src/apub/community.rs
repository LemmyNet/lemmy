use crate::apub::make_apub_endpoint;
use crate::db::community::Community;
use crate::db::community_view::CommunityFollowerView;
use crate::db::establish_connection;
use crate::to_datetime_utc;
use activitypub::{actor::Group, collection::UnorderedCollection, context};
use actix_web::body::Body;
use actix_web::web::Path;
use actix_web::HttpResponse;
use serde::Deserialize;

impl Community {
  pub fn as_group(&self) -> Group {
    let base_url = make_apub_endpoint("c", &self.name);

    let mut group = Group::default();

    group.object_props.set_context_object(context()).ok();
    group.object_props.set_id_string(base_url.to_string()).ok();
    group
      .object_props
      .set_name_string(self.name.to_owned())
      .ok();
    group
      .object_props
      .set_published_utctime(to_datetime_utc(self.published))
      .ok();
    if let Some(updated) = self.updated {
      group
        .object_props
        .set_updated_utctime(to_datetime_utc(updated))
        .ok();
    }

    if let Some(description) = &self.description {
      group
        .object_props
        .set_summary_string(description.to_string())
        .ok();
    }

    group
      .ap_actor_props
      .set_inbox_string(format!("{}/inbox", &base_url))
      .ok();
    group
      .ap_actor_props
      .set_outbox_string(format!("{}/outbox", &base_url))
      .ok();
    group
      .ap_actor_props
      .set_followers_string(format!("{}/followers", &base_url))
      .ok();

    group
  }

  pub fn followers_as_collection(&self) -> UnorderedCollection {
    let base_url = make_apub_endpoint("c", &self.name);

    let mut collection = UnorderedCollection::default();
    collection.object_props.set_context_object(context()).ok();
    collection.object_props.set_id_string(base_url).ok();

    let connection = establish_connection();
    //As we are an object, we validated that the community id was valid
    let community_followers = CommunityFollowerView::for_community(&connection, self.id).unwrap();

    let ap_followers = community_followers
      .iter()
      .map(|follower| make_apub_endpoint("u", &follower.user_name))
      .collect();

    collection
      .collection_props
      .set_items_string_vec(ap_followers)
      .unwrap();
    collection
  }
}

#[derive(Deserialize)]
pub struct CommunityQuery {
  community_name: String,
}

pub fn get_apub_community(info: Path<CommunityQuery>) -> HttpResponse<Body> {
  let connection = establish_connection();

  if let Ok(community) = Community::read_from_name(&connection, info.community_name.to_owned()) {
    HttpResponse::Ok()
      .content_type("application/activity+json")
      .body(serde_json::to_string(&community.as_group()).unwrap())
  } else {
    HttpResponse::NotFound().finish()
  }
}

pub fn get_apub_community_followers(info: Path<CommunityQuery>) -> HttpResponse<Body> {
  let connection = establish_connection();

  if let Ok(community) = Community::read_from_name(&connection, info.community_name.to_owned()) {
    HttpResponse::Ok()
      .content_type("application/activity+json")
      .body(serde_json::to_string(&community.followers_as_collection()).unwrap())
  } else {
    HttpResponse::NotFound().finish()
  }
}
