use crate::{
  extensions::context::lemmy_context,
  http::{create_apub_response, create_apub_tombstone_response},
  objects::ToApub,
  ActorType,
};
use activitystreams::{
  base::{AnyBase, BaseExt},
  collection::{CollectionExt, OrderedCollection, UnorderedCollection},
};
use actix_web::{body::Body, web, HttpResponse};
use lemmy_db_queries::source::{activity::Activity_, community::Community_};
use lemmy_db_schema::source::{activity::Activity, community::Community};
use lemmy_db_views_actor::community_follower_view::CommunityFollowerView;
use lemmy_structs::blocking;
use lemmy_utils::LemmyError;
use lemmy_websocket::LemmyContext;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct CommunityQuery {
  community_name: String,
}

/// Return the ActivityPub json representation of a local community over HTTP.
pub async fn get_apub_community_http(
  info: web::Path<CommunityQuery>,
  context: web::Data<LemmyContext>,
) -> Result<HttpResponse<Body>, LemmyError> {
  let community = blocking(context.pool(), move |conn| {
    Community::read_from_name(conn, &info.community_name)
  })
  .await??;

  if !community.deleted {
    let apub = community.to_apub(context.pool()).await?;

    Ok(create_apub_response(&apub))
  } else {
    Ok(create_apub_tombstone_response(&community.to_tombstone()?))
  }
}

/// Returns an empty followers collection, only populating the size (for privacy).
pub async fn get_apub_community_followers(
  info: web::Path<CommunityQuery>,
  context: web::Data<LemmyContext>,
) -> Result<HttpResponse<Body>, LemmyError> {
  let community = blocking(context.pool(), move |conn| {
    Community::read_from_name(&conn, &info.community_name)
  })
  .await??;

  let community_id = community.id;
  let community_followers = blocking(context.pool(), move |conn| {
    CommunityFollowerView::for_community(&conn, community_id)
  })
  .await??;

  let mut collection = UnorderedCollection::new();
  collection
    .set_many_contexts(lemmy_context()?)
    .set_id(community.get_followers_url()?)
    .set_total_items(community_followers.len() as u64);
  Ok(create_apub_response(&collection))
}

/// Returns the community outbox, which is populated by a maximum of 20 posts (but no other
/// activites like votes or comments).
pub async fn get_apub_community_outbox(
  info: web::Path<CommunityQuery>,
  context: web::Data<LemmyContext>,
) -> Result<HttpResponse<Body>, LemmyError> {
  let community = blocking(context.pool(), move |conn| {
    Community::read_from_name(&conn, &info.community_name)
  })
  .await??;

  let community_actor_id = community.actor_id.to_owned();
  let activities = blocking(context.pool(), move |conn| {
    Activity::read_community_outbox(conn, &community_actor_id)
  })
  .await??;

  let activities = activities
    .iter()
    .map(AnyBase::from_arbitrary_json)
    .collect::<Result<Vec<AnyBase>, serde_json::Error>>()?;
  let len = activities.len();
  let mut collection = OrderedCollection::new();
  collection
    .set_many_items(activities)
    .set_many_contexts(lemmy_context()?)
    .set_id(community.get_outbox_url()?)
    .set_total_items(len as u64);
  Ok(create_apub_response(&collection))
}

pub async fn get_apub_community_inbox(
  info: web::Path<CommunityQuery>,
  context: web::Data<LemmyContext>,
) -> Result<HttpResponse<Body>, LemmyError> {
  let community = blocking(context.pool(), move |conn| {
    Community::read_from_name(&conn, &info.community_name)
  })
  .await??;

  let mut collection = OrderedCollection::new();
  collection
    .set_id(format!("{}/inbox", community.actor_id).parse()?)
    .set_many_contexts(lemmy_context()?);
  Ok(create_apub_response(&collection))
}
