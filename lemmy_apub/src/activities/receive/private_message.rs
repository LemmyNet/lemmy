use crate::{
  activities::receive::verify_activity_domains_valid,
  check_is_apub_id_valid,
  fetcher::user::get_or_fetch_and_upsert_user,
  inbox::get_activity_to_and_cc,
  objects::FromApub,
  NoteExt,
};
use activitystreams::{
  activity::{ActorAndObjectRefExt, Create, Delete, Undo, Update},
  base::{AsBase, ExtendsExt},
  object::AsObject,
  public,
};
use anyhow::{anyhow, Context};
use lemmy_db_queries::source::private_message::PrivateMessage_;
use lemmy_db_schema::source::private_message::PrivateMessage;
use lemmy_db_views::private_message_view::PrivateMessageView;
use lemmy_structs::{blocking, user::PrivateMessageResponse};
use lemmy_utils::{location_info, LemmyError};
use lemmy_websocket::{messages::SendUserRoomMessage, LemmyContext, UserOperation};
use url::Url;

pub(crate) async fn receive_create_private_message(
  context: &LemmyContext,
  create: Create,
  expected_domain: Url,
  request_counter: &mut i32,
) -> Result<(), LemmyError> {
  check_private_message_activity_valid(&create, context, request_counter).await?;

  let note = NoteExt::from_any_base(
    create
      .object()
      .as_one()
      .context(location_info!())?
      .to_owned(),
  )?
  .context(location_info!())?;

  let private_message =
    PrivateMessage::from_apub(&note, context, expected_domain, request_counter).await?;

  let message = blocking(&context.pool(), move |conn| {
    PrivateMessageView::read(conn, private_message.id)
  })
  .await??;

  let res = PrivateMessageResponse {
    private_message_view: message,
  };

  let recipient_id = res.private_message_view.recipient.id;

  context.chat_server().do_send(SendUserRoomMessage {
    op: UserOperation::CreatePrivateMessage,
    response: res,
    recipient_id,
    websocket_id: None,
  });

  Ok(())
}

pub(crate) async fn receive_update_private_message(
  context: &LemmyContext,
  update: Update,
  expected_domain: Url,
  request_counter: &mut i32,
) -> Result<(), LemmyError> {
  check_private_message_activity_valid(&update, context, request_counter).await?;

  let object = update
    .object()
    .as_one()
    .context(location_info!())?
    .to_owned();
  let note = NoteExt::from_any_base(object)?.context(location_info!())?;

  let private_message =
    PrivateMessage::from_apub(&note, context, expected_domain, request_counter).await?;

  let private_message_id = private_message.id;
  let message = blocking(&context.pool(), move |conn| {
    PrivateMessageView::read(conn, private_message_id)
  })
  .await??;

  let res = PrivateMessageResponse {
    private_message_view: message,
  };

  let recipient_id = res.private_message_view.recipient.id;

  context.chat_server().do_send(SendUserRoomMessage {
    op: UserOperation::EditPrivateMessage,
    response: res,
    recipient_id,
    websocket_id: None,
  });

  Ok(())
}

pub(crate) async fn receive_delete_private_message(
  context: &LemmyContext,
  delete: Delete,
  private_message: PrivateMessage,
  request_counter: &mut i32,
) -> Result<(), LemmyError> {
  check_private_message_activity_valid(&delete, context, request_counter).await?;

  let deleted_private_message = blocking(context.pool(), move |conn| {
    PrivateMessage::update_deleted(conn, private_message.id, true)
  })
  .await??;

  let message = blocking(&context.pool(), move |conn| {
    PrivateMessageView::read(&conn, deleted_private_message.id)
  })
  .await??;

  let res = PrivateMessageResponse {
    private_message_view: message,
  };
  let recipient_id = res.private_message_view.recipient.id;
  context.chat_server().do_send(SendUserRoomMessage {
    op: UserOperation::EditPrivateMessage,
    response: res,
    recipient_id,
    websocket_id: None,
  });

  Ok(())
}

pub(crate) async fn receive_undo_delete_private_message(
  context: &LemmyContext,
  undo: Undo,
  expected_domain: &Url,
  private_message: PrivateMessage,
  request_counter: &mut i32,
) -> Result<(), LemmyError> {
  check_private_message_activity_valid(&undo, context, request_counter).await?;
  let object = undo.object().to_owned().one().context(location_info!())?;
  let delete = Delete::from_any_base(object)?.context(location_info!())?;
  verify_activity_domains_valid(&delete, expected_domain, true)?;
  check_private_message_activity_valid(&delete, context, request_counter).await?;

  let deleted_private_message = blocking(context.pool(), move |conn| {
    PrivateMessage::update_deleted(conn, private_message.id, false)
  })
  .await??;

  let message = blocking(&context.pool(), move |conn| {
    PrivateMessageView::read(&conn, deleted_private_message.id)
  })
  .await??;

  let res = PrivateMessageResponse {
    private_message_view: message,
  };
  let recipient_id = res.private_message_view.recipient.id;
  context.chat_server().do_send(SendUserRoomMessage {
    op: UserOperation::EditPrivateMessage,
    response: res,
    recipient_id,
    websocket_id: None,
  });

  Ok(())
}

async fn check_private_message_activity_valid<T, Kind>(
  activity: &T,
  context: &LemmyContext,
  request_counter: &mut i32,
) -> Result<(), LemmyError>
where
  T: AsBase<Kind> + AsObject<Kind> + ActorAndObjectRefExt,
{
  let to_and_cc = get_activity_to_and_cc(activity);
  if to_and_cc.len() != 1 {
    return Err(anyhow!("Private message can only be addressed to one user").into());
  }
  if to_and_cc.contains(&public()) {
    return Err(anyhow!("Private message cant be public").into());
  }
  let user_id = activity
    .actor()?
    .to_owned()
    .single_xsd_any_uri()
    .context(location_info!())?;
  check_is_apub_id_valid(&user_id)?;
  // check that the sender is a user, not a community
  get_or_fetch_and_upsert_user(&user_id, &context, request_counter).await?;

  Ok(())
}
