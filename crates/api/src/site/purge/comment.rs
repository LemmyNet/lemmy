use crate::Perform;
use actix_web::web::Data;
use lemmy_api_common::{
  context::LemmyContext,
  site::{PurgeComment, PurgeItemResponse},
  utils::{is_top_admin, local_user_view_from_jwt},
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

    // Only let the top admin purge an item
    is_top_admin(context.pool(), local_user_view.person.id).await?;

    let comment_id = data.comment_id;

    // Read the comment to get the post_id
    let comment = Comment::read(context.pool(), comment_id).await?;

    let post_id = comment.post_id;

    // TODO read comments for pictrs images and purge them

    Comment::delete(context.pool(), comment_id).await?;

    // Mod tables
    let reason = data.reason.clone();
    let form = AdminPurgeCommentForm {
      admin_person_id: local_user_view.person.id,
      reason,
      post_id,
    };

    AdminPurgeComment::create(context.pool(), &form).await?;

    Ok(PurgeItemResponse { success: true })
  }
}
