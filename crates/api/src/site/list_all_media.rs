use actix_web::web::{Data, Json, Query};
use lemmy_api_common::{
  context::LemmyContext,
  person::{ListMedia, ListMediaResponse},
  utils::is_admin,
};
use lemmy_db_schema::traits::PaginationCursorBuilder;
use lemmy_db_views::structs::{LocalImageView, LocalUserView};
use lemmy_utils::error::LemmyResult;

pub async fn list_all_media(
  data: Query<ListMedia>,
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> LemmyResult<Json<ListMediaResponse>> {
  // Only let admins view all media
  is_admin(&local_user_view)?;

  let cursor_data = if let Some(cursor) = &data.page_cursor {
    Some(LocalImageView::from_cursor(cursor, &mut context.pool()).await?)
  } else {
    None
  };

  let images =
    LocalImageView::get_all_paged(&mut context.pool(), cursor_data, data.page_back, data.limit)
      .await?;

  let next_page = images.last().map(PaginationCursorBuilder::to_cursor);
  let prev_page = images.first().map(PaginationCursorBuilder::to_cursor);

  Ok(Json(ListMediaResponse {
    images,
    next_page,
    prev_page,
  }))
}
