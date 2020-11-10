use crate::{
  activities::receive::{
    comment::{receive_create_comment, receive_update_comment},
    community::{
      receive_delete_community,
      receive_remove_community,
      receive_undo_delete_community,
      receive_undo_remove_community,
    },
    private_message::{
      receive_create_private_message,
      receive_delete_private_message,
      receive_undo_delete_private_message,
      receive_update_private_message,
    },
    receive_unhandled_activity,
    verify_activity_domains_valid,
  },
  check_is_apub_id_valid,
  fetcher::get_or_fetch_and_upsert_community,
  inbox::{
    get_activity_id,
    get_activity_to_and_cc,
    inbox_verify_http_signature,
    is_activity_already_known,
    is_addressed_to_public,
    receive_for_community::{
      receive_create_for_community,
      receive_delete_for_community,
      receive_dislike_for_community,
      receive_like_for_community,
      receive_remove_for_community,
      receive_undo_for_community,
      receive_update_for_community,
    },
  },
  insert_activity,
  ActorType,
};
use activitystreams::{
  activity::{Accept, ActorAndObject, Announce, Create, Delete, Follow, Undo, Update},
  base::AnyBase,
  prelude::*,
};
use actix_web::{web, HttpRequest, HttpResponse};
use anyhow::{anyhow, Context};
use diesel::NotFound;
use lemmy_db::{
  community::{Community, CommunityFollower, CommunityFollowerForm},
  private_message::PrivateMessage,
  user::User_,
  Followable,
};
use lemmy_structs::blocking;
use lemmy_utils::{location_info, LemmyError};
use lemmy_websocket::LemmyContext;
use log::debug;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use url::Url;

/// Allowed activities for user inbox.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub enum UserValidTypes {
  Accept,   // community accepted our follow request
  Create,   // create private message
  Update,   // edit private message
  Delete,   // private message or community deleted by creator
  Undo,     // private message or community restored
  Remove,   // community removed by admin
  Announce, // post, comment or vote in community
}

pub type UserAcceptedActivities = ActorAndObject<UserValidTypes>;

/// Handler for all incoming activities to user inboxes.
pub async fn user_inbox(
  request: HttpRequest,
  input: web::Json<UserAcceptedActivities>,
  path: web::Path<String>,
  context: web::Data<LemmyContext>,
) -> Result<HttpResponse, LemmyError> {
  let activity = input.into_inner();
  // First of all check the http signature
  let request_counter = &mut 0;
  let actor = inbox_verify_http_signature(&activity, &context, request, request_counter).await?;

  // Do nothing if we received the same activity before
  let activity_id = get_activity_id(&activity, &actor.actor_id()?)?;
  if is_activity_already_known(context.pool(), &activity_id).await? {
    return Ok(HttpResponse::Ok().finish());
  }

  // Check if the activity is actually meant for us
  let username = path.into_inner();
  let user = blocking(&context.pool(), move |conn| {
    User_::read_from_name(&conn, &username)
  })
  .await??;
  let to_and_cc = get_activity_to_and_cc(&activity)?;
  if !to_and_cc.contains(&&user.actor_id()?) {
    return Err(anyhow!("Activity delivered to wrong user").into());
  }

  insert_activity(&activity_id, activity.clone(), false, true, context.pool()).await?;

  debug!(
    "User {} received activity {:?} from {}",
    user.name,
    &activity.id_unchecked(),
    &actor.actor_id_str()
  );

  user_receive_message(
    activity.clone(),
    Some(user.clone()),
    actor.as_ref(),
    &context,
    request_counter,
  )
  .await
}

/// Receives Accept/Follow, Announce, private messages and community (undo) remove, (undo) delete
pub(crate) async fn user_receive_message(
  activity: UserAcceptedActivities,
  to_user: Option<User_>,
  actor: &dyn ActorType,
  context: &LemmyContext,
  request_counter: &mut i32,
) -> Result<HttpResponse, LemmyError> {
  // TODO: must be addressed to one or more local users, or to followers of a remote community

  // TODO: if it is addressed to community followers, check that at least one local user is following it

  let any_base = activity.clone().into_any_base()?;
  let kind = activity.kind().context(location_info!())?;
  let actor_url = actor.actor_id()?;
  match kind {
    UserValidTypes::Accept => {
      receive_accept(&context, any_base, actor, to_user.unwrap(), request_counter).await?;
    }
    UserValidTypes::Announce => {
      receive_announce(&context, any_base, actor, request_counter).await?
    }
    UserValidTypes::Create => {
      receive_create(&context, any_base, actor_url, request_counter).await?
    }
    UserValidTypes::Update => {
      receive_update(&context, any_base, actor_url, request_counter).await?
    }
    UserValidTypes::Delete => {
      receive_delete(context, any_base, &actor_url, request_counter).await?
    }
    UserValidTypes::Undo => receive_undo(context, any_base, &actor_url, request_counter).await?,
    UserValidTypes::Remove => receive_remove_community(&context, any_base, &actor_url).await?,
  };

  // TODO: would be logical to move websocket notification code here

  Ok(HttpResponse::Ok().finish())
}

/// Handle accepted follows.
async fn receive_accept(
  context: &LemmyContext,
  activity: AnyBase,
  actor: &dyn ActorType,
  user: User_,
  request_counter: &mut i32,
) -> Result<(), LemmyError> {
  let accept = Accept::from_any_base(activity)?.context(location_info!())?;
  verify_activity_domains_valid(&accept, &actor.actor_id()?, false)?;

  // TODO: we should check that we actually sent this activity, because the remote instance
  //       could just put a fake Follow
  let object = accept.object().to_owned().one().context(location_info!())?;
  let follow = Follow::from_any_base(object)?.context(location_info!())?;
  verify_activity_domains_valid(&follow, &user.actor_id()?, false)?;

  let community_uri = accept
    .actor()?
    .to_owned()
    .single_xsd_any_uri()
    .context(location_info!())?;

  let community =
    get_or_fetch_and_upsert_community(&community_uri, context, request_counter).await?;

  // Now you need to add this to the community follower
  let community_follower_form = CommunityFollowerForm {
    community_id: community.id,
    user_id: user.id,
  };

  // This will fail if they're already a follower
  blocking(&context.pool(), move |conn| {
    CommunityFollower::follow(conn, &community_follower_form).ok()
  })
  .await?;

  Ok(())
}

/// Takes an announce and passes the inner activity to the appropriate handler.
async fn receive_announce(
  context: &LemmyContext,
  activity: AnyBase,
  actor: &dyn ActorType,
  request_counter: &mut i32,
) -> Result<(), LemmyError> {
  let announce = Announce::from_any_base(activity)?.context(location_info!())?;
  verify_activity_domains_valid(&announce, &actor.actor_id()?, false)?;
  is_addressed_to_public(&announce)?;

  let kind = announce.object().as_single_kind_str();
  let inner_activity = announce
    .object()
    .to_owned()
    .one()
    .context(location_info!())?;

  let inner_id = inner_activity.id().context(location_info!())?.to_owned();
  check_is_apub_id_valid(&inner_id)?;
  if is_activity_already_known(context.pool(), &inner_id).await? {
    return Ok(());
  }

  match kind {
    Some("Create") => {
      receive_create_for_community(context, inner_activity, &inner_id, request_counter).await
    }
    Some("Update") => {
      receive_update_for_community(context, inner_activity, &inner_id, request_counter).await
    }
    Some("Like") => {
      receive_like_for_community(context, inner_activity, &inner_id, request_counter).await
    }
    Some("Dislike") => {
      receive_dislike_for_community(context, inner_activity, &inner_id, request_counter).await
    }
    Some("Delete") => receive_delete_for_community(context, inner_activity, &inner_id).await,
    Some("Remove") => receive_remove_for_community(context, inner_activity, &inner_id).await,
    Some("Undo") => {
      receive_undo_for_community(context, inner_activity, &inner_id, request_counter).await
    }
    _ => receive_unhandled_activity(inner_activity),
  }
}

async fn receive_create(
  context: &LemmyContext,
  activity: AnyBase,
  expected_domain: Url,
  request_counter: &mut i32,
) -> Result<(), LemmyError> {
  let create = Create::from_any_base(activity)?.context(location_info!())?;
  verify_activity_domains_valid(&create, &expected_domain, true)?;
  if is_addressed_to_public(&create).is_ok() {
    receive_create_comment(create, context, request_counter).await
  } else {
    receive_create_private_message(&context, create, expected_domain, request_counter).await
  }
}

async fn receive_update(
  context: &LemmyContext,
  activity: AnyBase,
  expected_domain: Url,
  request_counter: &mut i32,
) -> Result<(), LemmyError> {
  let update = Update::from_any_base(activity)?.context(location_info!())?;
  verify_activity_domains_valid(&update, &expected_domain, true)?;
  if is_addressed_to_public(&update).is_ok() {
    receive_update_comment(update, context, request_counter).await
  } else {
    receive_update_private_message(&context, update, expected_domain, request_counter).await
  }
}

async fn receive_delete(
  context: &LemmyContext,
  any_base: AnyBase,
  expected_domain: &Url,
  request_counter: &mut i32,
) -> Result<(), LemmyError> {
  use CommunityOrPrivateMessage::*;

  let delete = Delete::from_any_base(any_base.clone())?.context(location_info!())?;
  verify_activity_domains_valid(&delete, expected_domain, true)?;
  let object_uri = delete
    .object()
    .to_owned()
    .single_xsd_any_uri()
    .context(location_info!())?;

  match find_community_or_private_message_by_id(context, object_uri).await? {
    Community(c) => receive_delete_community(context, c).await,
    PrivateMessage(p) => receive_delete_private_message(context, delete, p, request_counter).await,
  }
}

async fn receive_undo(
  context: &LemmyContext,
  any_base: AnyBase,
  expected_domain: &Url,
  request_counter: &mut i32,
) -> Result<(), LemmyError> {
  use CommunityOrPrivateMessage::*;
  let undo = Undo::from_any_base(any_base)?.context(location_info!())?;
  verify_activity_domains_valid(&undo, expected_domain, true)?;

  let inner_activity = undo.object().to_owned().one().context(location_info!())?;
  let kind = inner_activity.kind_str();
  match kind {
    Some("Delete") => {
      let delete = Delete::from_any_base(inner_activity)?.context(location_info!())?;
      verify_activity_domains_valid(&delete, expected_domain, true)?;
      let object_uri = delete
        .object()
        .to_owned()
        .single_xsd_any_uri()
        .context(location_info!())?;
      match find_community_or_private_message_by_id(context, object_uri).await? {
        Community(c) => receive_undo_delete_community(context, undo, c, expected_domain).await,
        PrivateMessage(p) => {
          receive_undo_delete_private_message(context, undo, expected_domain, p, request_counter)
            .await
        }
      }
    }
    Some("Remove") => receive_undo_remove_community(context, undo, expected_domain).await,
    _ => receive_unhandled_activity(undo),
  }
}
enum CommunityOrPrivateMessage {
  Community(Community),
  PrivateMessage(PrivateMessage),
}

async fn find_community_or_private_message_by_id(
  context: &LemmyContext,
  apub_id: Url,
) -> Result<CommunityOrPrivateMessage, LemmyError> {
  let ap_id = apub_id.to_string();
  let community = blocking(context.pool(), move |conn| {
    Community::read_from_actor_id(conn, &ap_id)
  })
  .await?;
  if let Ok(c) = community {
    return Ok(CommunityOrPrivateMessage::Community(c));
  }

  let ap_id = apub_id.to_string();
  let private_message = blocking(context.pool(), move |conn| {
    PrivateMessage::read_from_apub_id(conn, &ap_id)
  })
  .await?;
  if let Ok(p) = private_message {
    return Ok(CommunityOrPrivateMessage::PrivateMessage(p));
  }

  return Err(NotFound.into());
}
