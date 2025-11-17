use activitypub_federation::config::Data;
use actix_web::web::Json;
use lemmy_api_utils::{
  context::LemmyContext,
  send_activity::{ActivityChannel, SendActivityData},
  utils::check_local_user_valid,
};
use lemmy_db_schema::source::private_message::{PrivateMessage, PrivateMessageUpdateForm};
use lemmy_db_views_local_user::LocalUserView;
use lemmy_db_views_private_message::{
  PrivateMessageView,
  api::{DeletePrivateMessage, PrivateMessageResponse},
};
use lemmy_diesel_utils::traits::Crud;
use lemmy_utils::error::{LemmyErrorType, LemmyResult};

pub async fn delete_private_message(
  data: Json<DeletePrivateMessage>,
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> LemmyResult<Json<PrivateMessageResponse>> {
  check_local_user_valid(&local_user_view)?;
  // Checking permissions
  let private_message_id = data.private_message_id;
  let orig_private_message = PrivateMessage::read(&mut context.pool(), private_message_id).await?;
  if local_user_view.person.id != orig_private_message.creator_id {
    Err(LemmyErrorType::EditPrivateMessageNotAllowed)?
  }

  // Doing the update
  let private_message_id = data.private_message_id;
  let deleted = data.deleted;
  let private_message = PrivateMessage::update(
    &mut context.pool(),
    private_message_id,
    &PrivateMessageUpdateForm {
      deleted: Some(deleted),
      ..Default::default()
    },
  )
  .await?;

  ActivityChannel::submit_activity(
    SendActivityData::DeletePrivateMessage(local_user_view.person, private_message, data.deleted),
    &context,
  )?;

  let view = PrivateMessageView::read(&mut context.pool(), private_message_id).await?;
  Ok(Json(PrivateMessageResponse {
    private_message_view: view,
  }))
}
