use crate::apub::{create_apub_response, make_apub_endpoint, EndpointType};
use crate::convert_datetime;
use crate::db::community::Community;
use crate::db::community_view::CommunityFollowerView;
use crate::db::establish_unpooled_connection;
use crate::db::post_view::{PostQueryBuilder, PostView};
use activitystreams::collection::apub::OrderedCollection;
use activitystreams::{
  actor::apub::Group, collection::apub::UnorderedCollection, context,
  object::properties::ObjectProperties,
};
use actix_web::body::Body;
use actix_web::web::Path;
use actix_web::HttpResponse;
use actix_web::{web, Result};
use diesel::r2d2::{ConnectionManager, Pool};
use diesel::PgConnection;
use failure::Error;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct CommunityQuery {
  community_name: String,
}

pub async fn get_apub_community(
  info: Path<CommunityQuery>,
  db: web::Data<Pool<ConnectionManager<PgConnection>>>,
) -> Result<HttpResponse<Body>, Error> {
  let community = Community::read_from_name(&&db.get()?, info.community_name.to_owned())?;
  let base_url = make_apub_endpoint(EndpointType::Community, &community.name);

  let mut group = Group::default();
  let oprops: &mut ObjectProperties = group.as_mut();

  oprops
    .set_context_xsd_any_uri(context())?
    .set_id(base_url.to_owned())?
    .set_name_xsd_string(community.title.to_owned())?
    .set_published(convert_datetime(community.published))?
    .set_attributed_to_xsd_any_uri(make_apub_endpoint(
      EndpointType::User,
      &community.creator_id.to_string(),
    ))?;

  if let Some(u) = community.updated.to_owned() {
    oprops.set_updated(convert_datetime(u))?;
  }
  if let Some(d) = community.description {
    oprops.set_summary_xsd_string(d)?;
  }

  group
    .ap_actor_props
    .set_inbox(format!("{}/inbox", &base_url))?
    .set_outbox(format!("{}/outbox", &base_url))?
    .set_followers(format!("{}/followers", &base_url))?;

  Ok(create_apub_response(serde_json::to_string(&group)?))
}

pub async fn get_apub_community_followers(
  info: Path<CommunityQuery>,
  db: web::Data<Pool<ConnectionManager<PgConnection>>>,
) -> Result<HttpResponse<Body>, Error> {
  let community = Community::read_from_name(&&db.get()?, info.community_name.to_owned())?;
  let base_url = make_apub_endpoint(EndpointType::Community, &community.name);

  let connection = establish_unpooled_connection();
  //As we are an object, we validated that the community id was valid
  let community_followers =
    CommunityFollowerView::for_community(&connection, community.id).unwrap();

  let mut collection = UnorderedCollection::default();
  let oprops: &mut ObjectProperties = collection.as_mut();
  oprops
    .set_context_xsd_any_uri(context())?
    .set_id(base_url)?;
  collection
    .collection_props
    .set_total_items(community_followers.len() as u64)?;
  Ok(create_apub_response(serde_json::to_string(&collection)?))
}

pub async fn get_apub_community_outbox(
  info: Path<CommunityQuery>,
  db: web::Data<Pool<ConnectionManager<PgConnection>>>,
) -> Result<HttpResponse<Body>, Error> {
  let community = Community::read_from_name(&&db.get()?, info.community_name.to_owned())?;
  let base_url = make_apub_endpoint(EndpointType::Community, &community.name);

  let connection = establish_unpooled_connection();
  //As we are an object, we validated that the community id was valid
  let community_posts: Vec<PostView> = PostQueryBuilder::create(&connection)
    .for_community_id(community.id)
    .list()
    .unwrap();

  let mut collection = OrderedCollection::default();
  let oprops: &mut ObjectProperties = collection.as_mut();
  oprops
    .set_context_xsd_any_uri(context())?
    .set_id(base_url)?;
  collection
    .collection_props
    .set_many_items_object_boxs(
      community_posts
        .iter()
        .map(|c| c.as_page().unwrap())
        .collect(),
    )?
    .set_total_items(community_posts.len() as u64)?;

  Ok(create_apub_response(serde_json::to_string(&collection)?))
}
