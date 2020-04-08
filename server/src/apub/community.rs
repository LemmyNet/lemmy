use crate::apub::puller::{fetch_remote_object, fetch_remote_user};
use crate::apub::*;
use crate::db::community::{Community, CommunityForm};
use crate::db::community_view::CommunityFollowerView;
use crate::db::establish_unpooled_connection;
use crate::db::post::Post;
use crate::db::user::User_;
use crate::db::Crud;
use crate::settings::Settings;
use crate::{convert_datetime, naive_now};
use activitystreams::actor::properties::ApActorProperties;
use activitystreams::collection::OrderedCollection;
use activitystreams::{
  actor::Group, collection::UnorderedCollection, context, ext::Extensible,
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
use url::Url;

#[derive(Deserialize)]
pub struct CommunityQuery {
  community_name: String,
}

pub async fn get_apub_community_list(
  db: web::Data<Pool<ConnectionManager<PgConnection>>>,
) -> Result<HttpResponse<Body>, Error> {
  // TODO: implement pagination
  let communities = Community::list_local(&db.get().unwrap())?
    .iter()
    .map(|c| c.as_group(&db.get().unwrap()))
    .collect::<Result<Vec<GroupExt>, Error>>()?;
  let mut collection = UnorderedCollection::default();
  let oprops: &mut ObjectProperties = collection.as_mut();
  oprops.set_context_xsd_any_uri(context())?.set_id(format!(
    "{}://{}/federation/communities",
    get_apub_protocol_string(),
    Settings::get().hostname
  ))?;

  collection
    .collection_props
    .set_total_items(communities.len() as u64)?
    .set_many_items_base_boxes(communities)?;
  Ok(create_apub_response(&collection))
}

impl Community {
  fn as_group(&self, conn: &PgConnection) -> Result<GroupExt, Error> {
    let base_url = make_apub_endpoint(EndpointType::Community, &self.name);

    let mut group = Group::default();
    let oprops: &mut ObjectProperties = group.as_mut();

    let creator = User_::read(conn, self.creator_id)?;
    oprops
      .set_context_xsd_any_uri(context())?
      .set_id(base_url.to_owned())?
      .set_name_xsd_string(self.name.to_owned())?
      .set_published(convert_datetime(self.published))?
      .set_attributed_to_xsd_any_uri(make_apub_endpoint(EndpointType::User, &creator.name))?;

    if let Some(u) = self.updated.to_owned() {
      oprops.set_updated(convert_datetime(u))?;
    }
    if let Some(d) = self.description.to_owned() {
      oprops.set_summary_xsd_string(d)?;
    }

    let mut actor_props = ApActorProperties::default();

    actor_props
      .set_preferred_username(self.title.to_owned())?
      .set_inbox(format!("{}/inbox", &base_url))?
      .set_outbox(format!("{}/outbox", &base_url))?
      .set_followers(format!("{}/followers", &base_url))?;

    Ok(group.extend(actor_props))
  }
}

impl CommunityForm {
  pub fn from_group(group: &GroupExt, conn: &PgConnection) -> Result<Self, Error> {
    let followers_uri = Url::parse(&group.extension.get_followers().unwrap().to_string())?;
    let outbox_uri = Url::parse(&group.extension.get_outbox().to_string())?;
    let _outbox = fetch_remote_object::<OrderedCollection>(&outbox_uri)?;
    let _followers = fetch_remote_object::<UnorderedCollection>(&followers_uri)?;
    let oprops = &group.base.object_props;
    let aprops = &group.extension;
    let apub_id = Url::parse(&oprops.get_attributed_to_xsd_any_uri().unwrap().to_string())?;
    let creator = fetch_remote_user(&apub_id, conn)?;
    Ok(CommunityForm {
      name: oprops.get_name_xsd_string().unwrap().to_string(),
      title: aprops.get_preferred_username().unwrap().to_string(),
      description: oprops.get_summary_xsd_string().map(|s| s.to_string()),
      category_id: 1,
      creator_id: creator.id,
      removed: None,
      published: oprops
        .get_published()
        .map(|u| u.as_ref().to_owned().naive_local()),
      updated: oprops
        .get_updated()
        .map(|u| u.as_ref().to_owned().naive_local()),
      deleted: None,
      nsfw: false,
      actor_id: oprops.get_id().unwrap().to_string(),
      local: false,
      private_key: None,
      public_key: None,
      last_refreshed_at: Some(naive_now()),
    })
  }
}

pub async fn get_apub_community_http(
  info: Path<CommunityQuery>,
  db: web::Data<Pool<ConnectionManager<PgConnection>>>,
) -> Result<HttpResponse<Body>, Error> {
  let community = Community::read_from_name(&&db.get()?, info.community_name.to_owned())?;
  let c = community.as_group(&db.get().unwrap())?;
  Ok(create_apub_response(&c))
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
  Ok(create_apub_response(&collection))
}

pub async fn get_apub_community_outbox(
  info: Path<CommunityQuery>,
  db: web::Data<Pool<ConnectionManager<PgConnection>>>,
) -> Result<HttpResponse<Body>, Error> {
  let community = Community::read_from_name(&&db.get()?, info.community_name.to_owned())?;
  let base_url = make_apub_endpoint(EndpointType::Community, &community.name);

  let conn = establish_unpooled_connection();
  //As we are an object, we validated that the community id was valid
  let community_posts: Vec<Post> = Post::list_for_community(&conn, community.id)?;

  let mut collection = OrderedCollection::default();
  let oprops: &mut ObjectProperties = collection.as_mut();
  oprops
    .set_context_xsd_any_uri(context())?
    .set_id(base_url)?;
  collection
    .collection_props
    .set_many_items_base_boxes(
      community_posts
        .iter()
        .map(|c| c.as_page(&conn).unwrap())
        .collect(),
    )?
    .set_total_items(community_posts.len() as u64)?;

  Ok(create_apub_response(&collection))
}
