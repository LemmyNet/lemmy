use actix_web::web::{Data, Json};
use lemmy_api_common::{
  context::LemmyContext,
  person::{CommentReplyResponse, MarkCommentReplyAsRead},
};
use lemmy_db_schema::{
  source::comment_reply::{CommentReply, CommentReplyUpdateForm},
  traits::Crud,
};
use lemmy_db_views::structs::LocalUserView;
use lemmy_db_views_actor::structs::CommentReplyView;
use lemmy_utils::error::{LemmyErrorExt, LemmyErrorType, LemmyResult};

#[tracing::instrument(skip(context))]
pub async fn mark_reply_as_read(
  data: Json<MarkCommentReplyAsRead>,
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> LemmyResult<Json<CommentReplyResponse>> {
  let comment_reply_id = data.comment_reply_id;
  let read_comment_reply = CommentReply::read(&mut context.pool(), comment_reply_id)
    .await?
    .ok_or(LemmyErrorType::CouldntFindCommentReply)?;

  if local_user_view.person.id != read_comment_reply.recipient_id {
    Err(LemmyErrorType::CouldntUpdateComment)?
  }

  let comment_reply_id = read_comment_reply.id;
  let read = Some(data.read);

  CommentReply::update(
    &mut context.pool(),
    comment_reply_id,
    &CommentReplyUpdateForm { read },
  )
  .await
  .with_lemmy_type(LemmyErrorType::CouldntUpdateComment)?;

  let comment_reply_id = read_comment_reply.id;
  let person_id = local_user_view.person.id;
  let comment_reply_view =
    CommentReplyView::read(&mut context.pool(), comment_reply_id, Some(person_id))
      .await?
      .ok_or(LemmyErrorType::CouldntFindCommentReply)?;

  Ok(Json(CommentReplyResponse { comment_reply_view }))
}
