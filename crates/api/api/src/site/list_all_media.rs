use actix_web::web::{Data, Json, Query};
use lemmy_api_utils::{context::LemmyContext, utils::is_admin};
use lemmy_db_views_local_image::{LocalImageView, api::ListMedia};
use lemmy_db_views_local_user::LocalUserView;
use lemmy_diesel_utils::pagination::PagedResponse;
use lemmy_utils::error::LemmyResult;

pub async fn list_all_media(
  Query(data): Query<ListMedia>,
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> LemmyResult<Json<PagedResponse<LocalImageView>>> {
  // Only let admins view all media
  is_admin(&local_user_view)?;

  let images =
    LocalImageView::get_all_paged(&mut context.pool(), data.page_cursor, data.limit).await?;

  Ok(Json(images))
}
