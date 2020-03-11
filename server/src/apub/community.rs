use crate::apub::group_wrapper::GroupHelper;
use crate::apub::make_apub_endpoint;
use crate::db::community::Community;
use crate::db::community_view::CommunityFollowerView;
use crate::db::establish_unpooled_connection;
use activitypub::{actor::Group, collection::UnorderedCollection, context};
use actix_web::body::Body;
use actix_web::web::Path;
use actix_web::HttpResponse;
use serde::Deserialize;
use serde_json::{Value};

impl Community {
  pub fn as_group(&self) -> Group {
    let base_url = make_apub_endpoint("c", &self.name);

    let mut group = Group::default();

    group.object_props.set_context_object(context()).ok();
    Group::set_id(&mut group, self.id);
    Group::set_title(&mut group, &self.title);
    Group::set_published(&mut group, self.published);
    Group::set_updated(&mut group, self.updated);
    Group::set_creator_id(&mut group, self.creator_id);

    Group::set_description(&mut group, &self.description);

    group.ap_actor_props.inbox = Value::String(format!("{}/inbox", &base_url));
    group.ap_actor_props.outbox = Value::String(format!("{}/outbox", &base_url));
    group.ap_actor_props.followers = Some(Value::String(format!("{}/followers", &base_url)));

    group
  }

  pub fn followers_as_collection(&self) -> UnorderedCollection {
    let base_url = make_apub_endpoint("c", &self.name);

    let mut collection = UnorderedCollection::default();
    collection.object_props.set_context_object(context()).ok();
    collection.object_props.set_id_string(base_url).ok();

    let connection = establish_unpooled_connection();
    //As we are an object, we validated that the community id was valid
    let community_followers = CommunityFollowerView::for_community(&connection, self.id).unwrap();

    collection
      .collection_props
      .set_total_items_u64(community_followers.len() as u64)
      .unwrap();
    collection
  }
}

#[derive(Deserialize)]
pub struct CommunityQuery {
  community_name: String,
}

pub async fn get_apub_community(info: Path<CommunityQuery>) -> HttpResponse<Body> {
  let connection = establish_unpooled_connection();

  if let Ok(community) = Community::read_from_name(&connection, info.community_name.to_owned()) {
    HttpResponse::Ok()
      .content_type("application/activity+json")
      .body(serde_json::to_string(&community.as_group()).unwrap())
  } else {
    HttpResponse::NotFound().finish()
  }
}

pub async fn get_apub_community_followers(info: Path<CommunityQuery>) -> HttpResponse<Body> {
  let connection = establish_unpooled_connection();

  if let Ok(community) = Community::read_from_name(&connection, info.community_name.to_owned()) {
    HttpResponse::Ok()
      .content_type("application/activity+json")
      .body(serde_json::to_string(&community.followers_as_collection()).unwrap())
  } else {
    HttpResponse::NotFound().finish()
  }
}
