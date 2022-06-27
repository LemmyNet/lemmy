use crate::{
  activity_lists::GroupInboxActivities,
  collections::{
    community_moderators::ApubCommunityModerators,
    community_outbox::ApubCommunityOutbox,
    CommunityContext,
  },
  generate_outbox_url,
  http::{create_apub_response, create_apub_tombstone_response, receive_lemmy_activity},
  local_instance,
  objects::{community::ApubCommunity, person::ApubPerson},
  protocol::collections::group_followers::GroupFollowers,
};
use activitypub_federation::{
  core::object_id::ObjectId,
  deser::context::WithContext,
  traits::ApubObject,
};
use actix_web::{web, HttpRequest, HttpResponse};
use lemmy_api_common::utils::blocking;
use lemmy_db_schema::{source::community::Community, traits::ApubActor};
use lemmy_utils::error::LemmyError;
use lemmy_websocket::LemmyContext;
use serde::Deserialize;

#[derive(Deserialize)]
pub(crate) struct CommunityQuery {
  community_name: String,
}

/// Return the ActivityPub json representation of a local community over HTTP.
#[tracing::instrument(skip_all)]
pub(crate) async fn get_apub_community_http(
  info: web::Path<CommunityQuery>,
  context: web::Data<LemmyContext>,
) -> Result<HttpResponse, LemmyError> {
  let community: ApubCommunity = blocking(context.pool(), move |conn| {
    Community::read_from_name(conn, &info.community_name, true)
  })
  .await??
  .into();

  if !community.deleted {
    let apub = community.into_apub(&**context).await?;

    Ok(create_apub_response(&apub))
  } else {
    Ok(create_apub_tombstone_response(community.actor_id.clone()))
  }
}

/// Handler for all incoming receive to community inboxes.
#[tracing::instrument(skip_all)]
pub async fn community_inbox(
  request: HttpRequest,
  payload: String,
  context: web::Data<LemmyContext>,
) -> Result<HttpResponse, LemmyError> {
  receive_lemmy_activity::<WithContext<GroupInboxActivities>, ApubPerson>(request, payload, context)
    .await
}

/// Returns an empty followers collection, only populating the size (for privacy).
pub(crate) async fn get_apub_community_followers(
  info: web::Path<CommunityQuery>,
  context: web::Data<LemmyContext>,
) -> Result<HttpResponse, LemmyError> {
  let community = blocking(context.pool(), move |conn| {
    Community::read_from_name(conn, &info.community_name, false)
  })
  .await??;
  let followers = GroupFollowers::new(community, &context).await?;
  Ok(create_apub_response(&followers))
}

/// Returns the community outbox, which is populated by a maximum of 20 posts (but no other
/// activites like votes or comments).
pub(crate) async fn get_apub_community_outbox(
  info: web::Path<CommunityQuery>,
  context: web::Data<LemmyContext>,
) -> Result<HttpResponse, LemmyError> {
  let community = blocking(context.pool(), move |conn| {
    Community::read_from_name(conn, &info.community_name, false)
  })
  .await??;
  let id = ObjectId::new(generate_outbox_url(&community.actor_id)?);
  let outbox_data = CommunityContext(community.into(), context.get_ref().clone());
  let outbox: ApubCommunityOutbox = id
    .dereference(&outbox_data, local_instance(&context), &mut 0)
    .await?;
  Ok(create_apub_response(&outbox.into_apub(&outbox_data).await?))
}

#[tracing::instrument(skip_all)]
pub(crate) async fn get_apub_community_moderators(
  info: web::Path<CommunityQuery>,
  context: web::Data<LemmyContext>,
) -> Result<HttpResponse, LemmyError> {
  let community: ApubCommunity = blocking(context.pool(), move |conn| {
    Community::read_from_name(conn, &info.community_name, false)
  })
  .await??
  .into();
  let id = ObjectId::new(generate_outbox_url(&community.actor_id)?);
  let outbox_data = CommunityContext(community, context.get_ref().clone());
  let moderators: ApubCommunityModerators = id
    .dereference(&outbox_data, local_instance(&context), &mut 0)
    .await?;
  Ok(create_apub_response(
    &moderators.into_apub(&outbox_data).await?,
  ))
}
