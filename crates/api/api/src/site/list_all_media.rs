use actix_web::web::{Data, Json, Query};
use lemmy_api_utils::{context::LemmyContext, utils::is_admin};
use lemmy_db_views_local_image::{
  LocalImageView,
  api::{ListMedia, ListMediaResponse},
};
use lemmy_db_views_local_user::LocalUserView;
use lemmy_utils::error::LemmyResult;

pub async fn list_all_media(
  Query(data): Query<ListMedia>,
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> LemmyResult<Json<ListMediaResponse>> {
  // Only let admins view all media
  is_admin(&local_user_view)?;

  let images =
    LocalImageView::get_all_paged(&mut context.pool(), data.page_cursor, data.limit).await?;

  Ok(Json(ListMediaResponse {
    images: images.data,
    next_page: images.next_page,
    prev_page: images.prev_page,
  }))
}
