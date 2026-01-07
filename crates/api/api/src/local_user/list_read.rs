use activitypub_federation::config::Data;
use actix_web::web::{Json, Query};
use lemmy_api_utils::context::LemmyContext;
use lemmy_db_views_local_user::LocalUserView;
use lemmy_db_views_person_content_combined::api::ListPersonRead;
use lemmy_db_views_post::PostView;
use lemmy_diesel_utils::pagination::PagedResponse;
use lemmy_utils::error::LemmyResult;

pub async fn list_person_read(
  Query(data): Query<ListPersonRead>,
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> LemmyResult<Json<PagedResponse<PostView>>> {
  let read = PostView::list_read(
    &mut context.pool(),
    &local_user_view.person,
    data.page_cursor,
    data.limit,
    None,
  )
  .await?;

  Ok(Json(read))
}
