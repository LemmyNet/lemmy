use actix_web::web::{Data, Json};
use lemmy_api_common::{
  context::LemmyContext,
  post::{MarkPostAsRead, PostResponse},
};
use lemmy_db_schema::source::post::PostRead;
use lemmy_db_views::structs::{LocalUserView, PostView};
use lemmy_utils::error::LemmyResult;

#[tracing::instrument(skip(context))]
pub async fn mark_post_as_read(
  data: Json<MarkPostAsRead>,
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> LemmyResult<Json<PostResponse>> {
  let person_id = local_user_view.person.id;
  let post_id = data.post_id;

  // Mark the post as read / unread
  if data.read {
    PostRead::mark_as_read(&mut context.pool(), post_id, person_id).await?;
  } else {
    PostRead::mark_as_unread(&mut context.pool(), post_id, person_id).await?;
  }
  let post_view = PostView::read(
    &mut context.pool(),
    post_id,
    Some(&local_user_view.local_user),
    false,
  )
  .await?;

  Ok(Json(PostResponse { post_view }))
}
