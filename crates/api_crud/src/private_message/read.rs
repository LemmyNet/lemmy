use actix_web::web::{Data, Json, Query};
use lemmy_api_common::{
  context::LemmyContext,
  private_message::{GetPrivateMessages, PrivateMessagesResponse},
};
use lemmy_db_views::{private_message_view::PrivateMessageQuery, structs::LocalUserView};
use lemmy_utils::error::LemmyResult;

#[tracing::instrument(skip(context))]
pub async fn get_private_message(
  data: Query<GetPrivateMessages>,
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> LemmyResult<Json<PrivateMessagesResponse>> {
  let person_id = local_user_view.person.id;

  let page = data.page;
  let limit = data.limit;
  let unread_only = data.unread_only.unwrap_or_default();
  let creator_id = data.creator_id;
  let messages = PrivateMessageQuery {
    page,
    limit,
    unread_only,
    creator_id,
  }
  .list(&mut context.pool(), person_id)
  .await?;

  Ok(Json(PrivateMessagesResponse {
    private_messages: messages,
  }))
}
