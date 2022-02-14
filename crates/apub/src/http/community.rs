use crate::{
  activities::{community::announce::GetCommunity, verify_person_in_community},
  activity_lists::GroupInboxActivities,
  collections::{
    community_moderators::ApubCommunityModerators,
    community_outbox::ApubCommunityOutbox,
    CommunityContext,
  },
  context::WithContext,
  generate_outbox_url,
  http::{
    create_apub_response,
    create_apub_tombstone_response,
    payload_to_string,
    receive_activity,
    ActivityCommonFields,
  },
  objects::community::ApubCommunity,
  protocol::{
    activities::community::announce::AnnounceActivity,
    collections::group_followers::GroupFollowers,
  },
};
use actix_web::{web, web::Payload, HttpRequest, HttpResponse};
use lemmy_api_common::blocking;
use lemmy_apub_lib::{object_id::ObjectId, traits::ApubObject};
use lemmy_db_schema::{source::community::Community, traits::ApubActor};
use lemmy_utils::LemmyError;
use lemmy_websocket::LemmyContext;
use serde::Deserialize;
use tracing::info;

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
    Community::read_from_name(conn, &info.community_name)
  })
  .await??
  .into();

  if !community.deleted {
    let apub = community.into_apub(&**context).await?;

    Ok(create_apub_response(&apub))
  } else {
    Ok(create_apub_tombstone_response(&community.to_tombstone()?))
  }
}

/// Handler for all incoming receive to community inboxes.
#[tracing::instrument(skip_all)]
pub async fn community_inbox(
  request: HttpRequest,
  payload: Payload,
  _path: web::Path<String>,
  context: web::Data<LemmyContext>,
) -> Result<HttpResponse, LemmyError> {
  let unparsed = payload_to_string(payload).await?;
  info!("Received community inbox activity {}", unparsed);
  let activity_data: ActivityCommonFields = serde_json::from_str(&unparsed)?;
  let activity = serde_json::from_str::<WithContext<GroupInboxActivities>>(&unparsed)?;

  receive_group_inbox(activity.inner(), activity_data, request, &context).await?;

  Ok(HttpResponse::Ok().finish())
}

pub(in crate::http) async fn receive_group_inbox(
  activity: GroupInboxActivities,
  activity_data: ActivityCommonFields,
  request: HttpRequest,
  context: &LemmyContext,
) -> Result<HttpResponse, LemmyError> {
  let actor_id = ObjectId::new(activity_data.actor.clone());
  let res = receive_activity(request, activity.clone(), activity_data, context).await?;

  if let GroupInboxActivities::AnnouncableActivities(announcable) = activity {
    // Ignore failures in get_community(). those happen because Delete/PrivateMessage is not in a
    // community, but looks identical to Delete/Post or Delete/Comment which are in a community.
    let community = announcable.get_community(context, &mut 0).await;
    if let Ok(community) = community {
      if community.local {
        verify_person_in_community(&actor_id, &community, context, &mut 0).await?;
        AnnounceActivity::send(*announcable, &community, context).await?;
      }
    }
  }

  Ok(res)
}

/// Returns an empty followers collection, only populating the size (for privacy).
pub(crate) async fn get_apub_community_followers(
  info: web::Path<CommunityQuery>,
  context: web::Data<LemmyContext>,
) -> Result<HttpResponse, LemmyError> {
  let community = blocking(context.pool(), move |conn| {
    Community::read_from_name(conn, &info.community_name)
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
    Community::read_from_name(conn, &info.community_name)
  })
  .await??;
  let id = ObjectId::new(generate_outbox_url(&community.actor_id)?);
  let outbox_data = CommunityContext(community.into(), context.get_ref().clone());
  let outbox: ApubCommunityOutbox = id
    .dereference(&outbox_data, context.client(), &mut 0)
    .await?;
  Ok(create_apub_response(&outbox.into_apub(&outbox_data).await?))
}

#[tracing::instrument(skip_all)]
pub(crate) async fn get_apub_community_moderators(
  info: web::Path<CommunityQuery>,
  context: web::Data<LemmyContext>,
) -> Result<HttpResponse, LemmyError> {
  let community: ApubCommunity = blocking(context.pool(), move |conn| {
    Community::read_from_name(conn, &info.community_name)
  })
  .await??
  .into();
  let id = ObjectId::new(generate_outbox_url(&community.actor_id)?);
  let outbox_data = CommunityContext(community, context.get_ref().clone());
  let moderators: ApubCommunityModerators = id
    .dereference(&outbox_data, context.client(), &mut 0)
    .await?;
  Ok(create_apub_response(
    &moderators.into_apub(&outbox_data).await?,
  ))
}
