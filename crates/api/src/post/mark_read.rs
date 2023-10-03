use actix_web::web::{Data, Json};
use lemmy_api_common::{
  context::LemmyContext,
  post::{MarkPostAsRead, PostResponse},
  utils,
};
use lemmy_db_views::structs::{LocalUserView, PostView};
use lemmy_utils::error::LemmyError;

#[tracing::instrument(skip(context))]
pub async fn mark_post_as_read(
  data: Json<MarkPostAsRead>,
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> Result<Json<PostResponse>, LemmyError> {
  let post_id = data.post_id;
  let person_id = local_user_view.person.id;

  // Mark the post as read / unread
  if data.read {
    utils::mark_post_as_read(person_id, post_id, &mut context.pool()).await?;
  } else {
    utils::mark_post_as_unread(person_id, post_id, &mut context.pool()).await?;
  }

  // Fetch it
  let post_view = PostView::read(&mut context.pool(), post_id, Some(person_id), false).await?;

  Ok(Json(PostResponse { post_view }))
}
