use crate::apub::make_apub_endpoint;
use crate::convert_datetime;
use crate::db::community::Community;
use crate::db::community_view::CommunityFollowerView;
use crate::db::establish_unpooled_connection;
use activitystreams::{
  actor::apub::Group, collection::apub::UnorderedCollection, context,
  object::properties::ObjectProperties,
};
use actix_web::body::Body;
use actix_web::web::Path;
use actix_web::HttpResponse;
use failure::Error;
use serde::Deserialize;

impl Community {
  pub fn as_group(&self) -> Result<Group, Error> {
    let base_url = make_apub_endpoint("c", &self.id);

    let mut group = Group::default();
    let oprops: &mut ObjectProperties = group.as_mut();

    oprops
      .set_context_xsd_any_uri(context())?
      .set_id(base_url.to_owned())?
      .set_name_xsd_string(self.title.to_owned())?
      .set_published(convert_datetime(self.published))?
      .set_attributed_to_xsd_any_uri(make_apub_endpoint("u", &self.creator_id))?;

    if let Some(u) = self.updated.to_owned() {
      oprops.set_updated(convert_datetime(u))?;
    }
    if let Some(d) = self.description.to_owned() {
      oprops.set_summary_xsd_string(d)?;
    }

    group
      .ap_actor_props
      .set_inbox(format!("{}/inbox", &base_url))?
      .set_outbox(format!("{}/outbox", &base_url))?
      .set_followers(format!("{}/followers", &base_url))?;

    Ok(group)
  }

  pub fn followers_as_collection(&self) -> Result<UnorderedCollection, Error> {
    let base_url = make_apub_endpoint("c", &self.name);

    let connection = establish_unpooled_connection();
    //As we are an object, we validated that the community id was valid
    let community_followers = CommunityFollowerView::for_community(&connection, self.id).unwrap();

    let mut collection = UnorderedCollection::default();
    let oprops: &mut ObjectProperties = collection.as_mut();
    oprops
      .set_context_xsd_any_uri(context())?
      .set_id(base_url)?;
    collection
      .collection_props
      .set_total_items(community_followers.len() as u64)?;
    Ok(collection)
  }
}

#[derive(Deserialize)]
pub struct CommunityQuery {
  community_name: String,
}

pub async fn get_apub_community(info: Path<CommunityQuery>) -> Result<HttpResponse<Body>, Error> {
  let connection = establish_unpooled_connection();

  if let Ok(community) = Community::read_from_name(&connection, info.community_name.to_owned()) {
    Ok(
      HttpResponse::Ok()
        .content_type("application/activity+json")
        .body(serde_json::to_string(&community.as_group()?).unwrap()),
    )
  } else {
    Ok(HttpResponse::NotFound().finish())
  }
}

pub async fn get_apub_community_followers(
  info: Path<CommunityQuery>,
) -> Result<HttpResponse<Body>, Error> {
  let connection = establish_unpooled_connection();

  if let Ok(community) = Community::read_from_name(&connection, info.community_name.to_owned()) {
    Ok(
      HttpResponse::Ok()
        .content_type("application/activity+json")
        .body(serde_json::to_string(&community.followers_as_collection()?).unwrap()),
    )
  } else {
    Ok(HttpResponse::NotFound().finish())
  }
}
