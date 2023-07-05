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
use lemmy_utils::error::LemmyError;

#[async_trait::async_trait(?Send)]
impl PerformCrud for DeleteComment {
  type Response = CommentResponse;

  #[tracing::instrument(skip(context))]
  async fn perform(&self, context: &Data<LemmyContext>) -> Result<CommentResponse, LemmyError> {
    let data: &DeleteComment = self;
    let local_user_view = local_user_view_from_jwt(&data.auth, context).await?;

    let comment_id = data.comment_id;
    let orig_comment = CommentView::read(context.pool(), comment_id, None).await?;

    // Dont delete it if its already been deleted.
    if orig_comment.comment.deleted == data.deleted {
      return Err(LemmyError::from_message("couldnt_update_comment"));
    }

    check_community_ban(
      local_user_view.person.id,
      orig_comment.community.id,
      context.pool(),
    )
    .await?;

    // Verify that only the creator can delete
    if local_user_view.person.id != orig_comment.creator.id {
      return Err(LemmyError::from_message("no_comment_edit_allowed"));
    }

    // Do the delete
    let deleted = data.deleted;
    let updated_comment = Comment::update(
      context.pool(),
      comment_id,
      &CommentUpdateForm::builder().deleted(Some(deleted)).build(),
    )
    .await
    .map_err(|e| LemmyError::from_error_message(e, "couldnt_update_comment"))?;

    let post_id = updated_comment.post_id;
    let post = Post::read(context.pool(), post_id).await?;
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
      context,
      updated_comment.id,
      Some(local_user_view),
      None,
      recipient_ids,
    )
    .await
  }
}
