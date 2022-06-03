use crate::Perform;
use actix_web::web::Data;
use lemmy_api_common::{
  site::{PurgeComment, PurgeItemResponse},
  utils::{blocking, get_local_user_view_from_jwt, is_admin},
};
use lemmy_db_schema::{
  source::{
    comment::Comment,
    moderator::{AdminPurgeComment, AdminPurgeCommentForm},
  },
  traits::Crud,
};
use lemmy_utils::{error::LemmyError, ConnectionId};
use lemmy_websocket::LemmyContext;

#[async_trait::async_trait(?Send)]
impl Perform for PurgeComment {
  type Response = PurgeItemResponse;

  #[tracing::instrument(skip(context, _websocket_id))]
  async fn perform(
    &self,
    context: &Data<LemmyContext>,
    _websocket_id: Option<ConnectionId>,
  ) -> Result<Self::Response, LemmyError> {
    let data: &Self = self;
    let local_user_view =
      get_local_user_view_from_jwt(&data.auth, context.pool(), context.secret()).await?;

    // Only let admins purge an item
    is_admin(&local_user_view)?;

    let comment_id = data.comment_id;

    // Read the comment to get the post_id
    let comment = blocking(context.pool(), move |conn| Comment::read(conn, comment_id)).await??;

    let post_id = comment.post_id;

    // TODO read comments for pictrs images and purge them

    blocking(context.pool(), move |conn| {
      Comment::delete(conn, comment_id)
    })
    .await??;

    // Mod tables
    let reason = data.reason.to_owned();
    let form = AdminPurgeCommentForm {
      admin_person_id: local_user_view.person.id,
      reason,
      post_id,
    };

    blocking(context.pool(), move |conn| {
      AdminPurgeComment::create(conn, &form)
    })
    .await??;

    Ok(PurgeItemResponse { success: true })
  }
}
