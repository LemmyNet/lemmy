use crate::{
  activities::{
    community::announce::{AnnouncableActivities, AnnounceActivity, GetCommunity},
    following::{follow::FollowCommunity, undo::UndoFollowCommunity},
    report::Report,
    verify_person_in_community,
  },
  collections::{
    community_followers::CommunityFollowers,
    community_moderators::ApubCommunityModerators,
    community_outbox::ApubCommunityOutbox,
    CommunityContext,
  },
  context::WithContext,
  fetcher::object_id::ObjectId,
  generate_outbox_url,
  http::{
    create_apub_response,
    create_apub_tombstone_response,
    payload_to_string,
    receive_activity,
  },
  objects::community::ApubCommunity,
};
use actix_web::{body::Body, web, web::Payload, HttpRequest, HttpResponse};
use lemmy_api_common::blocking;
use lemmy_apub_lib::{
  traits::{ActivityFields, ActivityHandler, ActorType, ApubObject},
  verify::verify_domains_match,
};
use lemmy_db_schema::source::community::Community;
use lemmy_utils::LemmyError;
use lemmy_websocket::LemmyContext;
use log::info;
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub(crate) struct CommunityQuery {
  community_name: String,
}

/// Return the ActivityPub json representation of a local community over HTTP.
pub(crate) async fn get_apub_community_http(
  info: web::Path<CommunityQuery>,
  context: web::Data<LemmyContext>,
) -> Result<HttpResponse<Body>, LemmyError> {
  let community: ApubCommunity = blocking(context.pool(), move |conn| {
    Community::read_from_name(conn, &info.community_name)
  })
  .await??
  .into();

  if !community.deleted {
    let apub = community.to_apub(&**context).await?;

    Ok(create_apub_response(&apub))
  } else {
    Ok(create_apub_tombstone_response(&community.to_tombstone()?))
  }
}

#[derive(Clone, Debug, Deserialize, Serialize, ActivityHandler, ActivityFields)]
#[serde(untagged)]
#[activity_handler(LemmyContext)]
pub enum GroupInboxActivities {
  FollowCommunity(FollowCommunity),
  UndoFollowCommunity(UndoFollowCommunity),
  AnnouncableActivities(AnnouncableActivities),
  Report(Report),
}

/// Handler for all incoming receive to community inboxes.
pub async fn community_inbox(
  request: HttpRequest,
  payload: Payload,
  _path: web::Path<String>,
  context: web::Data<LemmyContext>,
) -> Result<HttpResponse, LemmyError> {
  let unparsed = payload_to_string(payload).await?;
  info!("Received community inbox activity {}", unparsed);
  let activity = serde_json::from_str::<WithContext<GroupInboxActivities>>(&unparsed)?;

  receive_group_inbox(activity.inner(), request, &context).await?;

  Ok(HttpResponse::Ok().finish())
}

pub(in crate::http) async fn receive_group_inbox(
  activity: GroupInboxActivities,
  request: HttpRequest,
  context: &LemmyContext,
) -> Result<HttpResponse, LemmyError> {
  let res = receive_activity(request, activity.clone(), context).await;

  if let GroupInboxActivities::AnnouncableActivities(announcable) = activity {
    let community = announcable.get_community(context, &mut 0).await?;
    let actor_id = ObjectId::new(announcable.actor().clone());
    verify_domains_match(&community.actor_id(), announcable.id_unchecked())?;
    verify_person_in_community(&actor_id, &community, context, &mut 0).await?;
    if community.local {
      AnnounceActivity::send(announcable, &community, vec![], context).await?;
    }
  }

  res
}

/// Returns an empty followers collection, only populating the size (for privacy).
pub(crate) async fn get_apub_community_followers(
  info: web::Path<CommunityQuery>,
  context: web::Data<LemmyContext>,
) -> Result<HttpResponse<Body>, LemmyError> {
  let community = blocking(context.pool(), move |conn| {
    Community::read_from_name(conn, &info.community_name)
  })
  .await??;
  let followers = CommunityFollowers::new(community, &context).await?;
  Ok(create_apub_response(&followers))
}

/// Returns the community outbox, which is populated by a maximum of 20 posts (but no other
/// activites like votes or comments).
pub(crate) async fn get_apub_community_outbox(
  info: web::Path<CommunityQuery>,
  context: web::Data<LemmyContext>,
) -> Result<HttpResponse<Body>, LemmyError> {
  let community = blocking(context.pool(), move |conn| {
    Community::read_from_name(conn, &info.community_name)
  })
  .await??;
  let id = ObjectId::new(generate_outbox_url(&community.actor_id)?.into_inner());
  let outbox_data = CommunityContext(community.into(), context.get_ref().clone());
  let outbox: ApubCommunityOutbox = id.dereference(&outbox_data, &mut 0).await?;
  Ok(create_apub_response(&outbox.to_apub(&outbox_data).await?))
}

pub(crate) async fn get_apub_community_moderators(
  info: web::Path<CommunityQuery>,
  context: web::Data<LemmyContext>,
) -> Result<HttpResponse<Body>, LemmyError> {
  let community: ApubCommunity = blocking(context.pool(), move |conn| {
    Community::read_from_name(conn, &info.community_name)
  })
  .await??
  .into();
  let id = ObjectId::new(generate_outbox_url(&community.actor_id)?.into_inner());
  let outbox_data = CommunityContext(community, context.get_ref().clone());
  let moderators: ApubCommunityModerators = id.dereference(&outbox_data, &mut 0).await?;
  Ok(create_apub_response(
    &moderators.to_apub(&outbox_data).await?,
  ))
}
