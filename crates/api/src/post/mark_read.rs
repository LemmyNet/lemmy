use crate::Perform;
use actix_web::web::Data;
use lemmy_api_common::{
  context::LemmyContext,
  post::{MarkPostAsRead, PostResponse},
  utils::{local_user_view_from_jwt, mark_post_as_read, mark_post_as_unread},
};
use lemmy_db_views::structs::PostView;
use lemmy_utils::error::LemmyError;

#[async_trait::async_trait(?Send)]
impl Perform for MarkPostAsRead {
  type Response = PostResponse;

  #[tracing::instrument(skip(context))]
  async fn perform(&self, context: &Data<LemmyContext>) -> Result<Self::Response, LemmyError> {
    let data = self;
    let local_user_view = local_user_view_from_jwt(&data.auth, context).await?;

    let post_id = data.post_id;
    let person_id = local_user_view.person.id;

    // Mark the post as read / unread
    if data.read {
      mark_post_as_read(person_id, post_id, &mut context.pool()).await?;
    } else {
      mark_post_as_unread(person_id, post_id, &mut context.pool()).await?;
    }

    // Fetch it
    let post_view = PostView::read(&mut context.pool(), post_id, Some(person_id), false).await?;

    Ok(Self::Response { post_view })
  }
}
