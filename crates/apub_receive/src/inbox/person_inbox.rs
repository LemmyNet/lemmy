use crate::{
  activities::receive::{receive_unhandled_activity, verify_activity_domains_valid},
  inbox::{
    is_activity_already_known,
    is_addressed_to_community_followers,
    is_addressed_to_local_person,
    new_inbox_routing::{Activity, SharedInboxActivities},
    receive_for_community::receive_add_for_community,
    verify_is_addressed_to_public,
  },
};
use activitystreams::{
  activity::{ActorAndObject, Announce},
  base::AnyBase,
  prelude::*,
};
use actix_web::{web, HttpRequest, HttpResponse};
use anyhow::{anyhow, Context};
use lemmy_api_common::blocking;
use lemmy_apub::{check_is_apub_id_valid, get_activity_to_and_cc, ActorType};
use lemmy_apub_lib::{ReceiveActivity, VerifyActivity};
use lemmy_db_queries::Followable;
use lemmy_db_schema::source::{community::CommunityFollower, person::Person};
use lemmy_utils::{location_info, LemmyError};
use lemmy_websocket::LemmyContext;
use log::info;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use strum_macros::EnumString;

/// Allowed activities for person inbox.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub enum PersonValidTypes {
  Accept,   // community accepted our follow request
  Create,   // create private message
  Update,   // edit private message
  Delete,   // private message or community deleted by creator
  Undo,     // private message or community restored
  Remove,   // community removed by admin
  Announce, // post, comment or vote in community
}

pub type PersonAcceptedActivities = ActorAndObject<PersonValidTypes>;

/// Handler for all incoming activities to person inboxes.
pub async fn person_inbox(
  _request: HttpRequest,
  input: web::Json<Activity<SharedInboxActivities>>,
  _path: web::Path<String>,
  context: web::Data<LemmyContext>,
) -> Result<HttpResponse, LemmyError> {
  let activity = input.into_inner();
  activity.inner.verify(&context).await?;
  let request_counter = &mut 0;
  activity.inner.receive(&context, request_counter).await?;
  todo!()
  /*
  // First of all check the http signature
  let request_counter = &mut 0;
  let actor = inbox_verify_http_signature(&activity, &context, request, request_counter).await?;

  // Do nothing if we received the same activity before
  let activity_id = get_activity_id(&activity, &actor.actor_id())?;
  if is_activity_already_known(context.pool(), &activity_id).await? {
    return Ok(HttpResponse::Ok().finish());
  }

  // Check if the activity is actually meant for us
  let username = path.into_inner();
  let person = blocking(&context.pool(), move |conn| {
    Person::find_by_name(&conn, &username)
  })
  .await??;
  let to_and_cc = get_activity_to_and_cc(&activity);
  // TODO: we should also accept activities that are sent to community followers
  if !to_and_cc.contains(&&person.actor_id()) {
    return Err(anyhow!("Activity delivered to wrong person").into());
  }

  assert_activity_not_local(&activity)?;
  insert_activity(&activity_id, activity.clone(), false, true, context.pool()).await?;

  person_receive_message(
    activity.clone(),
    Some(person.clone()),
    actor.as_ref(),
    &context,
    request_counter,
  )
  .await
  */
}

/// Receives Accept/Follow, Announce, private messages and community (undo) remove, (undo) delete
pub(crate) async fn person_receive_message(
  activity: PersonAcceptedActivities,
  _to_person: Option<Person>,
  actor: &dyn ActorType,
  context: &LemmyContext,
  request_counter: &mut i32,
) -> Result<HttpResponse, LemmyError> {
  is_for_person_inbox(context, &activity).await?;

  info!(
    "User received activity {:?} from {}",
    &activity
      .id_unchecked()
      .context(location_info!())?
      .to_string(),
    &actor.actor_id().to_string()
  );

  let any_base = activity.clone().into_any_base()?;
  let kind = activity.kind().context(location_info!())?;
  match kind {
    PersonValidTypes::Accept => {}
    PersonValidTypes::Announce => {
      Box::pin(receive_announce(&context, any_base, actor, request_counter)).await?
    }
    PersonValidTypes::Create => {}
    PersonValidTypes::Update => {}
    PersonValidTypes::Delete => todo!(),
    PersonValidTypes::Undo => todo!(),
    PersonValidTypes::Remove => todo!(),
  };

  // TODO: would be logical to move websocket notification code here

  Ok(HttpResponse::Ok().finish())
}

/// Returns true if the activity is addressed directly to one or more local persons, or if it is
/// addressed to the followers collection of a remote community, and at least one local person follows
/// it.
async fn is_for_person_inbox(
  context: &LemmyContext,
  activity: &PersonAcceptedActivities,
) -> Result<(), LemmyError> {
  let to_and_cc = get_activity_to_and_cc(activity);
  // Check if it is addressed directly to any local person
  if is_addressed_to_local_person(&to_and_cc, context.pool()).await? {
    return Ok(());
  }

  // Check if it is addressed to any followers collection of a remote community, and that the
  // community has local followers.
  let community = is_addressed_to_community_followers(&to_and_cc, context.pool()).await?;
  if let Some(c) = community {
    let community_id = c.id;
    let has_local_followers = blocking(&context.pool(), move |conn| {
      CommunityFollower::has_local_followers(conn, community_id)
    })
    .await??;
    if c.local {
      return Err(
        anyhow!("Remote activity cant be addressed to followers of local community").into(),
      );
    }
    if has_local_followers {
      return Ok(());
    }
  }

  Err(anyhow!("Not addressed for any local person").into())
}

#[derive(EnumString)]
enum AnnouncableActivities {
  Create,
  Update,
  Like,
  Dislike,
  Delete,
  Remove,
  Undo,
  Add,
  Block,
}

/// Takes an announce and passes the inner activity to the appropriate handler.
pub async fn receive_announce(
  context: &LemmyContext,
  activity: AnyBase,
  actor: &dyn ActorType,
  request_counter: &mut i32,
) -> Result<(), LemmyError> {
  let announce = Announce::from_any_base(activity)?.context(location_info!())?;
  verify_activity_domains_valid(&announce, &actor.actor_id(), false)?;
  verify_is_addressed_to_public(&announce)?;

  let kind = announce
    .object()
    .as_single_kind_str()
    .and_then(|s| s.parse().ok());
  let inner_activity = announce
    .object()
    .to_owned()
    .one()
    .context(location_info!())?;

  let inner_id = inner_activity.id().context(location_info!())?.to_owned();
  check_is_apub_id_valid(&inner_id, false)?;
  if is_activity_already_known(context.pool(), &inner_id).await? {
    return Ok(());
  }

  use AnnouncableActivities::*;
  match kind {
    Some(Create) => todo!(),
    Some(Update) => todo!(),
    Some(Like) => todo!(),
    Some(Dislike) => todo!(),
    Some(Delete) => todo!(),
    Some(Remove) => todo!(),
    Some(Undo) => todo!(),
    Some(Add) => {
      receive_add_for_community(context, inner_activity, Some(announce), request_counter).await
    }
    Some(Block) => todo!(),
    _ => receive_unhandled_activity(inner_activity),
  }
}
