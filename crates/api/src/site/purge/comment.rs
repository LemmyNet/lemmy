use actix_web::web::{Data, Json};
use lemmy_api_common::{
  context::LemmyContext,
  site::{PurgeComment, PurgeItemResponse},
  utils::{is_admin, sanitize_html_opt},
};
use lemmy_db_schema::{
  source::{
    comment::Comment,
    moderator::{AdminPurgeComment, AdminPurgeCommentForm},
  },
  traits::Crud,
};
use lemmy_db_views::structs::LocalUserView;
use lemmy_utils::error::LemmyError;

#[tracing::instrument(skip(context))]
pub async fn purge_comment(
  data: Json<PurgeComment>,
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> Result<Json<PurgeItemResponse>, LemmyError> {
  // Only let admin purge an item
  is_admin(&local_user_view)?;

  let comment_id = data.comment_id;

  // Read the comment to get the post_id
  let comment = Comment::read(&mut context.pool(), comment_id).await?;

  let post_id = comment.post_id;

  // TODO read comments for pictrs images and purge them

  Comment::delete(&mut context.pool(), comment_id).await?;

  // Mod tables
  let reason = sanitize_html_opt(&data.reason);
  let form = AdminPurgeCommentForm {
    admin_person_id: local_user_view.person.id,
    reason,
    post_id,
  };

  AdminPurgeComment::create(&mut context.pool(), &form).await?;

  Ok(Json(PurgeItemResponse { success: true }))
}
