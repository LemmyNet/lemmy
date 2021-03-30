use crate::inbox::{
  assert_activity_not_local,
  community_inbox::{community_receive_message, CommunityAcceptedActivities},
  get_activity_id,
  inbox_verify_http_signature,
  is_activity_already_known,
  is_addressed_to_community_followers,
  is_addressed_to_local_person,
  person_inbox::{person_receive_message, PersonAcceptedActivities},
};
use activitystreams::{activity::ActorAndObject, prelude::*};
use actix_web::{web, HttpRequest, HttpResponse};
use anyhow::Context;
use lemmy_api_common::blocking;
use lemmy_apub::{get_activity_to_and_cc, insert_activity};
use lemmy_db_queries::{ApubObject, DbPool};
use lemmy_db_schema::source::community::Community;
use lemmy_utils::{location_info, LemmyError};
use lemmy_websocket::LemmyContext;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use url::Url;

/// Allowed activity types for shared inbox.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub enum ValidTypes {
  Create,
  Update,
  Like,
  Dislike,
  Delete,
  Undo,
  Remove,
  Announce,
  Add,
}

// TODO: this isnt entirely correct, cause some of these receive are not ActorAndObject,
//       but it still works due to the anybase conversion
pub type AcceptedActivities = ActorAndObject<ValidTypes>;

/// Handler for all incoming requests to shared inbox.
pub async fn shared_inbox(
  request: HttpRequest,
  input: web::Json<AcceptedActivities>,
  context: web::Data<LemmyContext>,
) -> Result<HttpResponse, LemmyError> {
  let activity = input.into_inner();
  // First of all check the http signature
  let request_counter = &mut 0;
  let actor = inbox_verify_http_signature(&activity, &context, request, request_counter).await?;

  // Do nothing if we received the same activity before
  let actor_id = actor.actor_id();
  let activity_id = get_activity_id(&activity, &actor_id)?;
  if is_activity_already_known(context.pool(), &activity_id).await? {
    return Ok(HttpResponse::Ok().finish());
  }

  assert_activity_not_local(&activity)?;
  // Log the activity, so we avoid receiving and parsing it twice. Note that this could still happen
  // if we receive the same activity twice in very quick succession.
  insert_activity(&activity_id, activity.clone(), false, true, context.pool()).await?;

  let activity_any_base = activity.clone().into_any_base()?;
  let mut res: Option<HttpResponse> = None;
  let to_and_cc = get_activity_to_and_cc(&activity);
  // Handle community first, so in case the sender is banned by the community, it will error out.
  // If we handled the person receive first, the activity would be inserted to the database before the
  // community could check for bans.
  // Note that an activity can be addressed to a community and to a person (or multiple persons) at the
  // same time. In this case we still only handle it once, to avoid duplicate websocket
  // notifications.
  let community = extract_local_community_from_destinations(&to_and_cc, context.pool()).await?;
  if let Some(community) = community {
    let community_activity = CommunityAcceptedActivities::from_any_base(activity_any_base.clone())?
      .context(location_info!())?;
    res = Some(
      Box::pin(community_receive_message(
        community_activity,
        community,
        actor.as_ref(),
        &context,
        request_counter,
      ))
      .await?,
    );
  } else if is_addressed_to_local_person(&to_and_cc, context.pool()).await? {
    let person_activity = PersonAcceptedActivities::from_any_base(activity_any_base.clone())?
      .context(location_info!())?;
    // `to_person` is only used for follow activities (which we dont receive here), so no need to pass
    // it in
    Box::pin(person_receive_message(
      person_activity,
      None,
      actor.as_ref(),
      &context,
      request_counter,
    ))
    .await?;
  } else if is_addressed_to_community_followers(&to_and_cc, context.pool())
    .await?
    .is_some()
  {
    let person_activity = PersonAcceptedActivities::from_any_base(activity_any_base.clone())?
      .context(location_info!())?;
    res = Some(
      Box::pin(person_receive_message(
        person_activity,
        None,
        actor.as_ref(),
        &context,
        request_counter,
      ))
      .await?,
    );
  }

  // If none of those, throw an error
  if let Some(r) = res {
    Ok(r)
  } else {
    Ok(HttpResponse::NotImplemented().finish())
  }
}

/// If `to_and_cc` contains the ID of a local community, return that community, otherwise return
/// None.
///
/// This doesnt handle the case where an activity is addressed to multiple communities (because
/// Lemmy doesnt generate such activities).
async fn extract_local_community_from_destinations(
  to_and_cc: &[Url],
  pool: &DbPool,
) -> Result<Option<Community>, LemmyError> {
  for url in to_and_cc {
    let url = url.to_owned();
    let community = blocking(&pool, move |conn| {
      Community::read_from_apub_id(&conn, &url.into())
    })
    .await?;
    if let Ok(c) = community {
      if c.local {
        return Ok(Some(c));
      }
    }
  }
  Ok(None)
}
