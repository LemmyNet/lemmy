use actix_web::web::{Data, Json, Query};
use lemmy_api_common::{
  context::LemmyContext,
  person::{ListMedia, ListMediaResponse},
};
use lemmy_db_schema::traits::PaginationCursorBuilder;
use lemmy_db_views_local_image::LocalImageView;
use lemmy_db_views_local_user::LocalUserView;
use lemmy_utils::error::LemmyResult;

pub async fn list_media(
  data: Query<ListMedia>,
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> LemmyResult<Json<ListMediaResponse>> {
  let cursor_data = if let Some(cursor) = &data.page_cursor {
    Some(LocalImageView::from_cursor(cursor, &mut context.pool()).await?)
  } else {
    None
  };

  let images = LocalImageView::get_all_paged_by_person_id(
    &mut context.pool(),
    local_user_view.person.id,
    cursor_data,
    data.page_back,
    data.limit,
  )
  .await?;

  let next_page = images.last().map(PaginationCursorBuilder::to_cursor);
  let prev_page = images.first().map(PaginationCursorBuilder::to_cursor);

  Ok(Json(ListMediaResponse {
    images,
    next_page,
    prev_page,
  }))
}
