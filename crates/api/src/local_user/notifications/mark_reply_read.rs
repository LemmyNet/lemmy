use crate::Perform;
use actix_web::web::Data;
use lemmy_api_common::{
  context::LemmyContext,
  person::{CommentReplyResponse, MarkCommentReplyAsRead},
  utils::local_user_view_from_jwt,
};
use lemmy_db_schema::{
  source::comment_reply::{CommentReply, CommentReplyUpdateForm},
  traits::Crud,
};
use lemmy_db_views_actor::structs::CommentReplyView;
use lemmy_utils::error::LemmyError;

#[async_trait::async_trait(?Send)]
impl Perform for MarkCommentReplyAsRead {
  type Response = CommentReplyResponse;

  #[tracing::instrument(skip(context))]
  async fn perform(
    &self,
    context: &Data<LemmyContext>,
  ) -> Result<CommentReplyResponse, LemmyError> {
    let data = self;
    let local_user_view = local_user_view_from_jwt(&data.auth, context).await?;

    let comment_reply_id = data.comment_reply_id;
    let read_comment_reply = CommentReply::read(context.pool(), comment_reply_id).await?;

    if local_user_view.person.id != read_comment_reply.recipient_id {
      return Err(LemmyError::from_message("couldnt_update_comment"));
    }

    let comment_reply_id = read_comment_reply.id;
    let read = Some(data.read);

    CommentReply::update(
      context.pool(),
      comment_reply_id,
      &CommentReplyUpdateForm { read },
    )
    .await
    .map_err(|e| LemmyError::from_error_message(e, "couldnt_update_comment"))?;

    let comment_reply_id = read_comment_reply.id;
    let person_id = local_user_view.person.id;
    let comment_reply_view =
      CommentReplyView::read(context.pool(), comment_reply_id, Some(person_id)).await?;

    Ok(CommentReplyResponse { comment_reply_view })
  }
}
