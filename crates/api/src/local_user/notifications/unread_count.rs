use actix_web::web::{Data, Json};
use lemmy_api_common::{context::LemmyContext, person::GetUnreadCountResponse};
use lemmy_db_views::structs::{LocalUserView, PrivateMessageView};
use lemmy_db_views_actor::structs::{CommentReplyView, PersonMentionView};
use lemmy_utils::error::LemmyResult;

#[tracing::instrument(skip(context))]
pub async fn unread_count(
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> LemmyResult<Json<GetUnreadCountResponse>> {
  let person_id = local_user_view.person.id;

  let replies =
    CommentReplyView::get_unread_replies(&mut context.pool(), &local_user_view.local_user).await?;

  let mentions =
    PersonMentionView::get_unread_mentions(&mut context.pool(), &local_user_view.local_user)
      .await?;

  let private_messages =
    PrivateMessageView::get_unread_messages(&mut context.pool(), person_id).await?;

  Ok(Json(GetUnreadCountResponse {
    replies,
    mentions,
    private_messages,
  }))
}
