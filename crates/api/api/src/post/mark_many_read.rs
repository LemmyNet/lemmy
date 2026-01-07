use actix_web::web::{Data, Json};
use lemmy_api_utils::context::LemmyContext;
use lemmy_db_schema::source::post::PostActions;
use lemmy_db_views_local_user::LocalUserView;
use lemmy_db_views_post::api::MarkManyPostsAsRead;
use lemmy_db_views_site::api::SuccessResponse;
use lemmy_utils::{error::LemmyResult, utils::validation::check_api_elements_count};

pub async fn mark_posts_as_read(
  Json(data): Json<MarkManyPostsAsRead>,
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> LemmyResult<Json<SuccessResponse>> {
  let post_ids = &data.post_ids;
  check_api_elements_count(post_ids.len())?;

  let person_id = local_user_view.person.id;

  // Mark the posts as read / unread
  if data.read {
    PostActions::mark_as_read(&mut context.pool(), person_id, post_ids).await?;
  } else {
    PostActions::mark_as_unread(&mut context.pool(), person_id, post_ids).await?;
  }

  Ok(Json(SuccessResponse::default()))
}
