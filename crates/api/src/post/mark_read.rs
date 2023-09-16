use actix_web::web::{Data, Json};
use lemmy_api_common::{
  context::LemmyContext,
  post::{MarkPostAsRead, PostResponse},
  utils,
  utils::local_user_view_from_jwt,
};
use lemmy_db_views::structs::PostView;
use lemmy_utils::error::LemmyError;

#[tracing::instrument(skip(context))]
pub async fn mark_post_as_read(
  data: Json<MarkPostAsRead>,
  context: Data<LemmyContext>,
) -> Result<Json<PostResponse>, LemmyError> {
  let local_user_view = local_user_view_from_jwt(&data.auth, &context).await?;

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
