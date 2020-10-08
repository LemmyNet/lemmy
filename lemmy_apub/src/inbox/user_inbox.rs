use crate::{
  check_is_apub_id_valid,
  extensions::signatures::verify,
  fetcher::{get_or_fetch_and_upsert_actor, get_or_fetch_and_upsert_community},
  insert_activity,
  FromApub,
};
use activitystreams::{
  activity::{Accept, ActorAndObject, Create, Delete, Undo, Update},
  base::AnyBase,
  object::Note,
  prelude::*,
};
use actix_web::{web, HttpRequest, HttpResponse};
use anyhow::Context;
use lemmy_db::{
  community::{CommunityFollower, CommunityFollowerForm},
  naive_now,
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

  let actor_uri = activity
    .actor()?
    .as_single_xsd_any_uri()
    .context(location_info!())?;
  debug!(
    "User {} inbox received activity {:?} from {}",
    username,
    &activity.id_unchecked(),
    &actor_uri
  );

  check_is_apub_id_valid(actor_uri)?;

  let actor = get_or_fetch_and_upsert_actor(actor_uri, &context).await?;
  verify(&request, actor.as_ref())?;

  let any_base = activity.clone().into_any_base()?;
  let kind = activity.kind().context(location_info!())?;
  let res = match kind {
    ValidTypes::Accept => receive_accept(any_base, username, &context).await,
    ValidTypes::Create => receive_create_private_message(any_base, &context).await,
    ValidTypes::Update => receive_update_private_message(any_base, &context).await,
    ValidTypes::Delete => receive_delete_private_message(any_base, &context).await,
    ValidTypes::Undo => receive_undo_delete_private_message(any_base, &context).await,
  };

  insert_activity(actor.user_id(), activity.clone(), false, context.pool()).await?;
  res
}

/// Handle accepted follows.
async fn receive_accept(
  activity: AnyBase,
  username: String,
  context: &LemmyContext,
) -> Result<HttpResponse, LemmyError> {
  let accept = Accept::from_any_base(activity)?.context(location_info!())?;
  let community_uri = accept
    .actor()?
    .to_owned()
    .single_xsd_any_uri()
    .context(location_info!())?;

  let community = get_or_fetch_and_upsert_community(&community_uri, context).await?;

  let user = blocking(&context.pool(), move |conn| {
    User_::read_from_name(conn, &username)
  })
  .await??;

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

  // TODO: make sure that we actually requested a follow
  Ok(HttpResponse::Ok().finish())
}

async fn receive_create_private_message(
  activity: AnyBase,
  context: &LemmyContext,
) -> Result<HttpResponse, LemmyError> {
  let create = Create::from_any_base(activity)?.context(location_info!())?;
  let note = Note::from_any_base(
    create
      .object()
      .as_one()
      .context(location_info!())?
      .to_owned(),
  )?
  .context(location_info!())?;

  let domain = Some(create.id_unchecked().context(location_info!())?.to_owned());
  let private_message = PrivateMessageForm::from_apub(&note, context, domain).await?;

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
  activity: AnyBase,
  context: &LemmyContext,
) -> Result<HttpResponse, LemmyError> {
  let update = Update::from_any_base(activity)?.context(location_info!())?;
  let note = Note::from_any_base(
    update
      .object()
      .as_one()
      .context(location_info!())?
      .to_owned(),
  )?
  .context(location_info!())?;

  let domain = Some(update.id_unchecked().context(location_info!())?.to_owned());
  let private_message_form = PrivateMessageForm::from_apub(&note, context, domain).await?;

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
  activity: AnyBase,
  context: &LemmyContext,
) -> Result<HttpResponse, LemmyError> {
  let delete = Delete::from_any_base(activity)?.context(location_info!())?;
  let note = Note::from_any_base(
    delete
      .object()
      .as_one()
      .context(location_info!())?
      .to_owned(),
  )?
  .context(location_info!())?;

  let domain = Some(delete.id_unchecked().context(location_info!())?.to_owned());
  let private_message_form = PrivateMessageForm::from_apub(&note, context, domain).await?;

  let private_message_ap_id = private_message_form.ap_id.context(location_info!())?;
  let private_message = blocking(&context.pool(), move |conn| {
    PrivateMessage::read_from_apub_id(conn, &private_message_ap_id)
  })
  .await??;

  let private_message_form = PrivateMessageForm {
    content: private_message_form.content,
    recipient_id: private_message.recipient_id,
    creator_id: private_message.creator_id,
    deleted: Some(true),
    read: None,
    ap_id: Some(private_message.ap_id),
    local: private_message.local,
    published: None,
    updated: Some(naive_now()),
  };

  let private_message_id = private_message.id;
  blocking(&context.pool(), move |conn| {
    PrivateMessage::update(conn, private_message_id, &private_message_form)
  })
  .await??;

  let private_message_id = private_message.id;
  let message = blocking(&context.pool(), move |conn| {
    PrivateMessageView::read(&conn, private_message_id)
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
  activity: AnyBase,
  context: &LemmyContext,
) -> Result<HttpResponse, LemmyError> {
  let undo = Undo::from_any_base(activity)?.context(location_info!())?;
  let delete = Delete::from_any_base(undo.object().as_one().context(location_info!())?.to_owned())?
    .context(location_info!())?;
  let note = Note::from_any_base(
    delete
      .object()
      .as_one()
      .context(location_info!())?
      .to_owned(),
  )?
  .context(location_info!())?;

  let domain = Some(undo.id_unchecked().context(location_info!())?.to_owned());
  let private_message = PrivateMessageForm::from_apub(&note, context, domain).await?;

  let private_message_ap_id = private_message
    .ap_id
    .as_ref()
    .context(location_info!())?
    .clone();
  let private_message_id = blocking(&context.pool(), move |conn| {
    PrivateMessage::read_from_apub_id(conn, &private_message_ap_id).map(|pm| pm.id)
  })
  .await??;

  let private_message_form = PrivateMessageForm {
    content: private_message.content,
    recipient_id: private_message.recipient_id,
    creator_id: private_message.creator_id,
    deleted: Some(false),
    read: None,
    ap_id: private_message.ap_id,
    local: private_message.local,
    published: None,
    updated: Some(naive_now()),
  };

  blocking(&context.pool(), move |conn| {
    PrivateMessage::update(conn, private_message_id, &private_message_form)
  })
  .await??;

  let message = blocking(&context.pool(), move |conn| {
    PrivateMessageView::read(&conn, private_message_id)
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
