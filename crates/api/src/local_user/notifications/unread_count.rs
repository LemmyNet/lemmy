use actix_web::web::{Data, Json, Query};
use lemmy_api_common::{
  context::LemmyContext,
  person::{GetUnreadCount, GetUnreadCountResponse},
  utils::local_user_view_from_jwt,
};
use lemmy_db_views::structs::PrivateMessageView;
use lemmy_db_views_actor::structs::{CommentReplyView, PersonMentionView};
use lemmy_utils::error::LemmyError;

#[tracing::instrument(skip(context))]
pub async fn unread_count(
  data: Query<GetUnreadCount>,
  context: Data<LemmyContext>,
) -> Result<Json<GetUnreadCountResponse>, LemmyError> {
  let local_user_view = local_user_view_from_jwt(&data.auth, &context).await?;

  let person_id = local_user_view.person.id;

  let replies = CommentReplyView::get_unread_replies(&mut context.pool(), person_id).await?;

  let mentions = PersonMentionView::get_unread_mentions(&mut context.pool(), person_id).await?;

  let private_messages =
    PrivateMessageView::get_unread_messages(&mut context.pool(), person_id).await?;

  Ok(Json(GetUnreadCountResponse {
    replies,
    mentions,
    private_messages,
  }))
}
