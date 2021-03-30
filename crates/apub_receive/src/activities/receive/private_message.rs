use crate::activities::receive::verify_activity_domains_valid;
use activitystreams::{
  activity::{ActorAndObjectRefExt, Create, Delete, Undo, Update},
  base::{AsBase, ExtendsExt},
  object::AsObject,
  public,
};
use anyhow::{anyhow, Context};
use lemmy_api_common::{blocking, person::PrivateMessageResponse};
use lemmy_apub::{
  check_is_apub_id_valid,
  fetcher::person::get_or_fetch_and_upsert_person,
  get_activity_to_and_cc,
  objects::FromApub,
  NoteExt,
};
use lemmy_db_queries::source::private_message::PrivateMessage_;
use lemmy_db_schema::source::private_message::PrivateMessage;
use lemmy_db_views::{local_user_view::LocalUserView, private_message_view::PrivateMessageView};
use lemmy_utils::{location_info, LemmyError};
use lemmy_websocket::{messages::SendUserRoomMessage, LemmyContext, UserOperationCrud};
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
    PrivateMessage::from_apub(&note, context, expected_domain, request_counter, false).await?;

  let message = blocking(&context.pool(), move |conn| {
    PrivateMessageView::read(conn, private_message.id)
  })
  .await??;

  let res = PrivateMessageResponse {
    private_message_view: message,
  };

  // Send notifications to the local recipient, if one exists
  let recipient_id = res.private_message_view.recipient.id;
  let local_recipient_id = blocking(context.pool(), move |conn| {
    LocalUserView::read_person(conn, recipient_id)
  })
  .await??
  .local_user
  .id;

  context.chat_server().do_send(SendUserRoomMessage {
    op: UserOperationCrud::CreatePrivateMessage,
    response: res,
    local_recipient_id,
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
    PrivateMessage::from_apub(&note, context, expected_domain, request_counter, false).await?;

  let private_message_id = private_message.id;
  let message = blocking(&context.pool(), move |conn| {
    PrivateMessageView::read(conn, private_message_id)
  })
  .await??;

  let res = PrivateMessageResponse {
    private_message_view: message,
  };

  let recipient_id = res.private_message_view.recipient.id;
  let local_recipient_id = blocking(context.pool(), move |conn| {
    LocalUserView::read_person(conn, recipient_id)
  })
  .await??
  .local_user
  .id;

  context.chat_server().do_send(SendUserRoomMessage {
    op: UserOperationCrud::EditPrivateMessage,
    response: res,
    local_recipient_id,
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
  let local_recipient_id = blocking(context.pool(), move |conn| {
    LocalUserView::read_person(conn, recipient_id)
  })
  .await??
  .local_user
  .id;

  context.chat_server().do_send(SendUserRoomMessage {
    op: UserOperationCrud::EditPrivateMessage,
    response: res,
    local_recipient_id,
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
  let local_recipient_id = blocking(context.pool(), move |conn| {
    LocalUserView::read_person(conn, recipient_id)
  })
  .await??
  .local_user
  .id;

  context.chat_server().do_send(SendUserRoomMessage {
    op: UserOperationCrud::EditPrivateMessage,
    response: res,
    local_recipient_id,
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
    return Err(anyhow!("Private message can only be addressed to one person").into());
  }
  if to_and_cc.contains(&public()) {
    return Err(anyhow!("Private message cant be public").into());
  }
  let person_id = activity
    .actor()?
    .to_owned()
    .single_xsd_any_uri()
    .context(location_info!())?;
  check_is_apub_id_valid(&person_id)?;
  // check that the sender is a person, not a community
  get_or_fetch_and_upsert_person(&person_id, &context, request_counter).await?;

  Ok(())
}
