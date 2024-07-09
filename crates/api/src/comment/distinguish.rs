use actix_web::web::{Data, Json};
use lemmy_api_common::{
  comment::{CommentResponse, DistinguishComment},
  context::LemmyContext,
  utils::{check_community_mod_action, check_community_user_action},
};
use lemmy_db_schema::{
  source::comment::{Comment, CommentUpdateForm},
  traits::Crud,
};
use lemmy_db_views::structs::{CommentView, LocalUserView};
use lemmy_utils::error::{LemmyErrorExt, LemmyErrorType, LemmyResult};

#[tracing::instrument(skip(context))]
pub async fn distinguish_comment(
  data: Json<DistinguishComment>,
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> LemmyResult<Json<CommentResponse>> {
  let orig_comment = CommentView::read(
    &mut context.pool(),
    data.comment_id,
    Some(&local_user_view.local_user),
  )
  .await?
  .ok_or(LemmyErrorType::CouldntFindComment)?;

  check_community_user_action(
    &local_user_view.person,
    orig_comment.community.id,
    &mut context.pool(),
  )
  .await?;

  // Verify that only the creator can distinguish
  if local_user_view.person.id != orig_comment.creator.id {
    Err(LemmyErrorType::NoCommentEditAllowed)?
  }

  // Verify that only a mod or admin can distinguish a comment
  check_community_mod_action(
    &local_user_view.person,
    orig_comment.community.id,
    false,
    &mut context.pool(),
  )
  .await?;

  // Update the Comment
  let form = CommentUpdateForm {
    distinguished: Some(data.distinguished),
    ..Default::default()
  };
  Comment::update(&mut context.pool(), data.comment_id, &form)
    .await
    .with_lemmy_type(LemmyErrorType::CouldntUpdateComment)?;

  let comment_view = CommentView::read(
    &mut context.pool(),
    data.comment_id,
    Some(&local_user_view.local_user),
  )
  .await?
  .ok_or(LemmyErrorType::CouldntFindComment)?;

  Ok(Json(CommentResponse {
    comment_view,
    recipient_ids: Vec::new(),
  }))
}
