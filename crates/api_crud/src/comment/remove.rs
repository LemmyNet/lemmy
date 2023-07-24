use crate::PerformCrud;
use actix_web::web::Data;
use lemmy_api_common::{
  build_response::{build_comment_response, send_local_notifs},
  comment::{CommentResponse, RemoveComment},
  context::LemmyContext,
  utils::{check_community_ban, is_mod_or_admin, local_user_view_from_jwt},
};
use lemmy_db_schema::{
  source::{
    comment::{Comment, CommentUpdateForm},
    moderator::{ModRemoveComment, ModRemoveCommentForm},
    post::Post,
  },
  traits::Crud,
};
use lemmy_db_views::structs::CommentView;
use lemmy_utils::error::{LemmyError, LemmyErrorExt, LemmyErrorType};

#[async_trait::async_trait(?Send)]
impl PerformCrud for RemoveComment {
  type Response = CommentResponse;

  #[tracing::instrument(skip(context))]
  async fn perform(&self, context: &Data<LemmyContext>) -> Result<CommentResponse, LemmyError> {
    let data: &RemoveComment = self;
    let local_user_view = local_user_view_from_jwt(&data.auth, context).await?;

    let comment_id = data.comment_id;
    let orig_comment = CommentView::read(&mut context.pool(), comment_id, None).await?;

    check_community_ban(
      local_user_view.person.id,
      orig_comment.community.id,
      &mut context.pool(),
    )
    .await?;

    // Verify that only a mod or admin can remove
    is_mod_or_admin(
      &mut context.pool(),
      local_user_view.person.id,
      orig_comment.community.id,
    )
    .await?;

    // Do the remove
    let removed = data.removed;
    let updated_comment = Comment::update(
      &mut context.pool(),
      comment_id,
      &CommentUpdateForm::builder().removed(Some(removed)).build(),
    )
    .await
    .with_lemmy_type(LemmyErrorType::CouldntUpdateComment)?;

    // Mod tables
    let form = ModRemoveCommentForm {
      mod_person_id: local_user_view.person.id,
      comment_id: data.comment_id,
      removed: Some(removed),
      reason: data.reason.clone(),
    };
    ModRemoveComment::create(&mut context.pool(), form).await?;

    let post_id = updated_comment.post_id;
    let post = Post::read(&mut context.pool(), post_id).await?;
    let recipient_ids = send_local_notifs(
      vec![],
      &updated_comment,
      &local_user_view.person.clone(),
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
