use actix_web::web::{Data, Json};
use lemmy_api_common::{context::LemmyContext, person::GetRepliesResponse};
use lemmy_db_schema::source::{
  comment_reply::CommentReply,
  person_mention::PersonMention,
  private_message::PrivateMessage,
};
use lemmy_db_views::structs::LocalUserView;
use lemmy_utils::error::{LemmyErrorExt, LemmyErrorType, LemmyResult};

#[tracing::instrument(skip(context))]
pub async fn mark_all_notifications_read(
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> LemmyResult<Json<GetRepliesResponse>> {
  let person_id = local_user_view.person.id;

  // Mark all comment_replies as read
  CommentReply::mark_all_as_read(&mut context.pool(), person_id)
    .await
    .with_lemmy_type(LemmyErrorType::CouldntUpdateComment)?;

  // Mark all user mentions as read
  PersonMention::mark_all_as_read(&mut context.pool(), person_id)
    .await
    .with_lemmy_type(LemmyErrorType::CouldntUpdateComment)?;

  // Mark all private_messages as read
  PrivateMessage::mark_all_as_read(&mut context.pool(), person_id)
    .await
    .with_lemmy_type(LemmyErrorType::CouldntUpdatePrivateMessage)?;

  Ok(Json(GetRepliesResponse { replies: vec![] }))
}
