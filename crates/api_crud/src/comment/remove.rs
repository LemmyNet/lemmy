use crate::PerformCrud;
use actix_web::web::Data;
use lemmy_api_common::{
  comment::{CommentResponse, RemoveComment},
  context::LemmyContext,
  utils::{check_community_ban, get_local_user_view_from_jwt, is_mod_or_admin},
  websocket::UserOperationCrud,
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
use lemmy_utils::{error::LemmyError, ConnectionId};

#[async_trait::async_trait(?Send)]
impl PerformCrud for RemoveComment {
  type Response = CommentResponse;

  #[tracing::instrument(skip(context, websocket_id))]
  async fn perform(
    &self,
    context: &Data<LemmyContext>,
    websocket_id: Option<ConnectionId>,
  ) -> Result<CommentResponse, LemmyError> {
    let data: &RemoveComment = self;
    let local_user_view =
      get_local_user_view_from_jwt(&data.auth, context.pool(), context.secret()).await?;

    let comment_id = data.comment_id;
    let orig_comment = CommentView::read(context.pool(), comment_id, None).await?;

    check_community_ban(
      local_user_view.person.id,
      orig_comment.community.id,
      context.pool(),
    )
    .await?;

    // Verify that only a mod or admin can remove
    is_mod_or_admin(
      context.pool(),
      local_user_view.person.id,
      orig_comment.community.id,
    )
    .await?;

    // Do the remove
    let removed = data.removed;
    let updated_comment = Comment::update(
      context.pool(),
      comment_id,
      &CommentUpdateForm::builder().removed(Some(removed)).build(),
    )
    .await
    .map_err(|e| LemmyError::from_error_message(e, "couldnt_update_comment"))?;

    // Mod tables
    let form = ModRemoveCommentForm {
      mod_person_id: local_user_view.person.id,
      comment_id: data.comment_id,
      removed: Some(removed),
      reason: data.reason.clone(),
    };
    ModRemoveComment::create(context.pool(), &form).await?;

    let post_id = updated_comment.post_id;
    let post = Post::read(context.pool(), post_id).await?;
    let recipient_ids = context
      .send_local_notifs(
        vec![],
        &updated_comment,
        &local_user_view.person.clone(),
        &post,
        false,
      )
      .await?;

    let res = context
      .send_comment_ws_message(
        &UserOperationCrud::RemoveComment,
        data.comment_id,
        websocket_id,
        None, // TODO maybe this might clear other forms
        Some(local_user_view.person.id),
        recipient_ids,
      )
      .await?;

    Ok(res)
  }
}
