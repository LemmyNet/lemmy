use crate::PerformCrud;
use actix_web::web::Data;
use lemmy_api_common::{
  build_response::{build_comment_response, send_local_notifs},
  comment::{CommentResponse, DeleteComment},
  context::LemmyContext,
  utils::{check_community_ban, local_user_view_from_jwt},
};
use lemmy_db_schema::{
  source::{
    comment::{Comment, CommentUpdateForm},
    post::Post,
  },
  traits::Crud,
};
use lemmy_db_views::structs::CommentView;
use lemmy_utils::error::{LemmyError, LemmyErrorExt, LemmyErrorType};
use std::ops::Deref;

#[async_trait::async_trait(?Send)]
impl PerformCrud for DeleteComment {
  type Response = CommentResponse;

  #[tracing::instrument(skip(context))]
  async fn perform(&self, context: &Data<LemmyContext>) -> Result<CommentResponse, LemmyError> {
    let data: &DeleteComment = self;
    let local_user_view = local_user_view_from_jwt(&data.auth, context).await?;

    let comment_id = data.comment_id;
    let orig_comment = CommentView::read(&mut context.pool(), comment_id, None).await?;

    // Dont delete it if its already been deleted.
    if orig_comment.comment.deleted == data.deleted {
      return Err(LemmyErrorType::CouldntUpdateComment)?;
    }

    check_community_ban(
      local_user_view.person.id,
      orig_comment.community.id,
      &mut context.pool(),
    )
    .await?;

    // Verify that only the creator can delete
    if local_user_view.person.id != orig_comment.creator.id {
      return Err(LemmyErrorType::NoCommentEditAllowed)?;
    }

    // Do the delete
    let deleted = data.deleted;
    let updated_comment = Comment::update(
      &mut context.pool(),
      comment_id,
      &CommentUpdateForm::builder().deleted(Some(deleted)).build(),
    )
    .await
    .with_lemmy_type(LemmyErrorType::CouldntUpdateComment)?;

    let post_id = updated_comment.post_id;
    let post = Post::read(&mut context.pool(), post_id).await?;
    let recipient_ids = send_local_notifs(
      vec![],
      &updated_comment,
      &local_user_view.person,
      &post,
      false,
      context,
    )
    .await?;

    build_comment_response(
      context.deref(),
      updated_comment.id,
      Some(local_user_view),
      None,
      recipient_ids,
    )
    .await
  }
}
