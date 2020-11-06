use crate::{
  activities::receive::verify_activity_domains_valid,
  check_is_apub_id_valid,
  extensions::signatures::verify_signature,
  fetcher::{get_or_fetch_and_upsert_actor, get_or_fetch_and_upsert_community},
  inbox::{get_activity_id, is_activity_already_known},
  insert_activity,
  ActorType,
  FromApub,
};
use activitystreams::{
  activity::{Accept, ActorAndObject, Create, Delete, Follow, Undo, Update},
  base::AnyBase,
  object::Note,
  prelude::*,
};
use actix_web::{web, HttpRequest, HttpResponse};
use anyhow::{anyhow, Context};
use lemmy_db::{
  community::{CommunityFollower, CommunityFollowerForm},
  private_message::{PrivateMessage, PrivateMessageForm},
  private_message_view::PrivateMessageView,
  user::User_,
  Crud,
  Followable,
};
use lemmy_structs::{blocking, user::PrivateMessageResponse};
use lemmy_utils::{location_info, LemmyError};
use lemmy_websocket::{messages::SendUserRoomMessage, LemmyContext, UserOperation};
use log::debug;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

/// Allowed activities for user inbox.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub enum ValidTypes {
  Accept,
  Create,
  Update,
  Delete,
  Undo,
}

pub type AcceptedActivities = ActorAndObject<ValidTypes>;

/// Handler for all incoming activities to user inboxes.
pub async fn user_inbox(
  request: HttpRequest,
  input: web::Json<AcceptedActivities>,
  path: web::Path<String>,
  context: web::Data<LemmyContext>,
) -> Result<HttpResponse, LemmyError> {
  let activity = input.into_inner();
  let username = path.into_inner();
  let user = blocking(&context.pool(), move |conn| {
    User_::read_from_name(&conn, &username)
  })
  .await??;

  let to = activity
    .to()
    .context(location_info!())?
    .to_owned()
    .single_xsd_any_uri();
  if Some(user.actor_id()?) != to {
    return Err(anyhow!("Activity delivered to wrong user").into());
  }

  let actor_uri = activity
    .actor()?
    .as_single_xsd_any_uri()
    .context(location_info!())?;
  debug!(
    "User {} inbox received activity {:?} from {}",
    user.name,
    &activity.id_unchecked(),
    &actor_uri
  );

  check_is_apub_id_valid(actor_uri)?;

  let request_counter = &mut 0;
  let actor = get_or_fetch_and_upsert_actor(actor_uri, &context, request_counter).await?;
  verify_signature(&request, actor.as_ref())?;

  let activity_id = get_activity_id(&activity, actor_uri)?;
  if is_activity_already_known(context.pool(), &activity_id).await? {
    return Ok(HttpResponse::Ok().finish());
  }

  let any_base = activity.clone().into_any_base()?;
  let kind = activity.kind().context(location_info!())?;
  let res = match kind {
    ValidTypes::Accept => {
      receive_accept(&context, any_base, actor.as_ref(), user, request_counter).await
    }
    ValidTypes::Create => {
      receive_create_private_message(&context, any_base, actor.as_ref(), request_counter).await
    }
    ValidTypes::Update => {
      receive_update_private_message(&context, any_base, actor.as_ref(), request_counter).await
    }
    ValidTypes::Delete => receive_delete_private_message(&context, any_base, actor.as_ref()).await,
    ValidTypes::Undo => {
      receive_undo_delete_private_message(&context, any_base, actor.as_ref()).await
    }
  };

  insert_activity(&activity_id, activity.clone(), false, true, context.pool()).await?;
  res
}

/// Handle accepted follows.
async fn receive_accept(
  context: &LemmyContext,
  activity: AnyBase,
  actor: &dyn ActorType,
  user: User_,
  request_counter: &mut i32,
) -> Result<HttpResponse, LemmyError> {
  let accept = Accept::from_any_base(activity)?.context(location_info!())?;
  verify_activity_domains_valid(&accept, actor.actor_id()?, false)?;

  // TODO: we should check that we actually sent this activity, because the remote instance
  //       could just put a fake Follow
  let object = accept.object().to_owned().one().context(location_info!())?;
  let follow = Follow::from_any_base(object)?.context(location_info!())?;
  verify_activity_domains_valid(&follow, user.actor_id()?, false)?;

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

  Ok(HttpResponse::Ok().finish())
}

async fn receive_create_private_message(
  context: &LemmyContext,
  activity: AnyBase,
  actor: &dyn ActorType,
  request_counter: &mut i32,
) -> Result<HttpResponse, LemmyError> {
  let create = Create::from_any_base(activity)?.context(location_info!())?;
  verify_activity_domains_valid(&create, actor.actor_id()?, true)?;

  let note = Note::from_any_base(
    create
      .object()
      .as_one()
      .context(location_info!())?
      .to_owned(),
  )?
  .context(location_info!())?;

  let private_message =
    PrivateMessageForm::from_apub(&note, context, Some(actor.actor_id()?), request_counter).await?;

  let inserted_private_message = blocking(&context.pool(), move |conn| {
    PrivateMessage::create(conn, &private_message)
  })
  .await??;

  let message = blocking(&context.pool(), move |conn| {
    PrivateMessageView::read(conn, inserted_private_message.id)
  })
  .await??;

  let res = PrivateMessageResponse { message };

  let recipient_id = res.message.recipient_id;

  context.chat_server().do_send(SendUserRoomMessage {
    op: UserOperation::CreatePrivateMessage,
    response: res,
    recipient_id,
    websocket_id: None,
  });

  Ok(HttpResponse::Ok().finish())
}

async fn receive_update_private_message(
  context: &LemmyContext,
  activity: AnyBase,
  actor: &dyn ActorType,
  request_counter: &mut i32,
) -> Result<HttpResponse, LemmyError> {
  let update = Update::from_any_base(activity)?.context(location_info!())?;
  verify_activity_domains_valid(&update, actor.actor_id()?, true)?;

  let object = update
    .object()
    .as_one()
    .context(location_info!())?
    .to_owned();
  let note = Note::from_any_base(object)?.context(location_info!())?;

  let private_message_form =
    PrivateMessageForm::from_apub(&note, context, Some(actor.actor_id()?), request_counter).await?;

  let private_message_ap_id = private_message_form
    .ap_id
    .as_ref()
    .context(location_info!())?
    .clone();
  let private_message = blocking(&context.pool(), move |conn| {
    PrivateMessage::read_from_apub_id(conn, &private_message_ap_id)
  })
  .await??;

  let private_message_id = private_message.id;
  blocking(&context.pool(), move |conn| {
    PrivateMessage::update(conn, private_message_id, &private_message_form)
  })
  .await??;

  let private_message_id = private_message.id;
  let message = blocking(&context.pool(), move |conn| {
    PrivateMessageView::read(conn, private_message_id)
  })
  .await??;

  let res = PrivateMessageResponse { message };

  let recipient_id = res.message.recipient_id;

  context.chat_server().do_send(SendUserRoomMessage {
    op: UserOperation::EditPrivateMessage,
    response: res,
    recipient_id,
    websocket_id: None,
  });

  Ok(HttpResponse::Ok().finish())
}

async fn receive_delete_private_message(
  context: &LemmyContext,
  activity: AnyBase,
  actor: &dyn ActorType,
) -> Result<HttpResponse, LemmyError> {
  let delete = Delete::from_any_base(activity)?.context(location_info!())?;
  verify_activity_domains_valid(&delete, actor.actor_id()?, true)?;

  let private_message_id = delete
    .object()
    .to_owned()
    .single_xsd_any_uri()
    .context(location_info!())?;
  let private_message = blocking(context.pool(), move |conn| {
    PrivateMessage::read_from_apub_id(conn, private_message_id.as_str())
  })
  .await??;
  let deleted_private_message = blocking(context.pool(), move |conn| {
    PrivateMessage::update_deleted(conn, private_message.id, true)
  })
  .await??;

  let message = blocking(&context.pool(), move |conn| {
    PrivateMessageView::read(&conn, deleted_private_message.id)
  })
  .await??;

  let res = PrivateMessageResponse { message };
  let recipient_id = res.message.recipient_id;
  context.chat_server().do_send(SendUserRoomMessage {
    op: UserOperation::EditPrivateMessage,
    response: res,
    recipient_id,
    websocket_id: None,
  });

  Ok(HttpResponse::Ok().finish())
}

async fn receive_undo_delete_private_message(
  context: &LemmyContext,
  activity: AnyBase,
  actor: &dyn ActorType,
) -> Result<HttpResponse, LemmyError> {
  let undo = Undo::from_any_base(activity)?.context(location_info!())?;
  verify_activity_domains_valid(&undo, actor.actor_id()?, true)?;
  let object = undo.object().to_owned().one().context(location_info!())?;
  let delete = Delete::from_any_base(object)?.context(location_info!())?;
  verify_activity_domains_valid(&delete, actor.actor_id()?, true)?;

  let private_message_id = delete
    .object()
    .to_owned()
    .single_xsd_any_uri()
    .context(location_info!())?;
  let private_message = blocking(context.pool(), move |conn| {
    PrivateMessage::read_from_apub_id(conn, private_message_id.as_str())
  })
  .await??;
  let deleted_private_message = blocking(context.pool(), move |conn| {
    PrivateMessage::update_deleted(conn, private_message.id, false)
  })
  .await??;

  let message = blocking(&context.pool(), move |conn| {
    PrivateMessageView::read(&conn, deleted_private_message.id)
  })
  .await??;

  let res = PrivateMessageResponse { message };
  let recipient_id = res.message.recipient_id;
  context.chat_server().do_send(SendUserRoomMessage {
    op: UserOperation::EditPrivateMessage,
    response: res,
    recipient_id,
    websocket_id: None,
  });

  Ok(HttpResponse::Ok().finish())
}
