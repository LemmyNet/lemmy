use crate::Perform;
use actix_web::web::Data;
use lemmy_api_common::{
  context::LemmyContext,
  site::{PurgeComment, PurgeItemResponse},
  utils::{is_admin, local_user_view_from_jwt, sanitize_html_opt},
};
use lemmy_db_schema::{
  source::{
    comment::Comment,
    moderator::{AdminPurgeComment, AdminPurgeCommentForm},
  },
  traits::Crud,
};
use lemmy_utils::error::LemmyError;

#[async_trait::async_trait(?Send)]
impl Perform for PurgeComment {
  type Response = PurgeItemResponse;

  #[tracing::instrument(skip(context))]
  async fn perform(&self, context: &Data<LemmyContext>) -> Result<Self::Response, LemmyError> {
    let data: &Self = self;
    let local_user_view = local_user_view_from_jwt(&data.auth, context).await?;

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

    Ok(PurgeItemResponse { success: true })
  }
}
