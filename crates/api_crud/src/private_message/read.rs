use actix_web::web::{Data, Json, Query};
use lemmy_api_common::{
  context::LemmyContext,
  private_message::{GetPrivateMessages, PrivateMessagesResponse},
  utils::local_user_view_from_jwt,
};
use lemmy_db_views::private_message_view::PrivateMessageQuery;
use lemmy_utils::error::LemmyError;

#[tracing::instrument(skip(context))]
pub async fn get_private_message(
  data: Query<GetPrivateMessages>,
  context: Data<LemmyContext>,
) -> Result<Json<PrivateMessagesResponse>, LemmyError> {
  let local_user_view = local_user_view_from_jwt(data.auth.as_ref(), &context).await?;
  let person_id = local_user_view.person.id;

  let page = data.page;
  let limit = data.limit;
  let unread_only = data.unread_only;
  let creator_id = data.creator_id;
  let mut messages = PrivateMessageQuery {
    page,
    limit,
    unread_only,
    creator_id,
  }
  .list(&mut context.pool(), person_id)
  .await?;

  // Messages sent by ourselves should be marked as read. The `read` column in database is only
  // for the recipient, and shouldnt be exposed to sender.
  messages.iter_mut().for_each(|pmv| {
    if pmv.creator.id == person_id {
      pmv.private_message.read = true
    }
  });

  Ok(Json(PrivateMessagesResponse {
    private_messages: messages,
  }))
}
