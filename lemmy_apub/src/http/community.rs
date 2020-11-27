use crate::{
  extensions::context::lemmy_context,
  http::{create_apub_response, create_apub_tombstone_response},
  ActorType,
  ToApub,
};
use activitystreams::{
  base::{AnyBase, BaseExt, ExtendsExt},
  collection::{CollectionExt, OrderedCollection, UnorderedCollection},
};
use actix_web::{body::Body, web, HttpResponse};
use lemmy_db::{community::Community, community_view::CommunityFollowerView, post::Post};
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

  let community_id = community.id;
  let posts = blocking(context.pool(), move |conn| {
    Post::list_for_community(conn, community_id)
  })
  .await??;

  let mut pages: Vec<AnyBase> = vec![];
  for p in posts {
    pages.push(p.to_apub(context.pool()).await?.into_any_base()?);
  }

  let len = pages.len();
  let mut collection = OrderedCollection::new();
  collection
    .set_many_items(pages)
    .set_many_contexts(lemmy_context()?)
    .set_id(community.get_outbox_url()?)
    .set_total_items(len as u64);
  Ok(create_apub_response(&collection))
}
