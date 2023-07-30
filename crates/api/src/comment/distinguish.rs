use actix_web::web::{Data, Json};
use lemmy_api_common::{
  comment::{CommentResponse, DistinguishComment},
  context::LemmyContext,
  utils::{check_community_ban, is_mod_or_admin, local_user_view_from_jwt},
};
use lemmy_db_schema::{
  source::comment::{Comment, CommentUpdateForm},
  traits::Crud,
};
use lemmy_db_views::structs::CommentView;
use lemmy_utils::error::{LemmyError, LemmyErrorExt, LemmyErrorType};

#[tracing::instrument(skip(context))]
pub async fn distinguish_comment(
  data: Json<DistinguishComment>,
  context: Data<LemmyContext>,
) -> Result<Json<CommentResponse>, LemmyError> {
  let local_user_view = local_user_view_from_jwt(&data.auth, &context).await?;

  let comment_id = data.comment_id;
  let orig_comment = CommentView::read(&mut context.pool(), comment_id, None).await?;

  check_community_ban(
    local_user_view.person.id,
    orig_comment.community.id,
    &mut context.pool(),
  )
  .await?;

  // Verify that only a mod or admin can distinguish a comment
  is_mod_or_admin(
    &mut context.pool(),
    local_user_view.person.id,
    orig_comment.community.id,
  )
  .await?;

  // Update the Comment
  let comment_id = data.comment_id;
  let form = CommentUpdateForm::builder()
    .distinguished(Some(data.distinguished))
    .build();
  Comment::update(&mut context.pool(), comment_id, &form)
    .await
    .with_lemmy_type(LemmyErrorType::CouldntUpdateComment)?;

  let comment_id = data.comment_id;
  let person_id = local_user_view.person.id;
  let comment_view = CommentView::read(&mut context.pool(), comment_id, Some(person_id)).await?;

  Ok(Json(CommentResponse {
    comment_view,
    recipient_ids: Vec::new(),
    form_id: None,
  }))
}
