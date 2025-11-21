use actix_web::web::{Data, Json, Query};
use lemmy_api_utils::context::LemmyContext;
use lemmy_db_views_local_image::{LocalImageView, api::ListMedia};
use lemmy_db_views_local_user::LocalUserView;
use lemmy_diesel_utils::pagination::PagedResponse;
use lemmy_utils::error::LemmyResult;

pub async fn list_media(
  Query(data): Query<ListMedia>,
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> LemmyResult<Json<PagedResponse<LocalImageView>>> {
  let images = LocalImageView::get_all_paged_by_person_id(
    &mut context.pool(),
    local_user_view.person.id,
    data.page_cursor,
    data.limit,
  )
  .await?;
  Ok(Json(images))
}
