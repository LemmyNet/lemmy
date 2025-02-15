use actix_web::web::{Data, Json, Query};
use lemmy_api_common::{
  context::LemmyContext,
  person::{ListMedia, ListMediaResponse},
};
use lemmy_db_schema::{
  source::images::LocalImage,
  traits::{PageCursorBuilder, PageCursorReader},
};
use lemmy_db_views::structs::{LocalImageView, LocalUserView};
use lemmy_utils::error::LemmyResult;

pub async fn list_media(
  data: Query<ListMedia>,
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> LemmyResult<Json<ListMediaResponse>> {
  let cursor_data = if let Some(cursor) = &data.page_cursor {
    Some(LocalImage::from_cursor(cursor, &mut context.pool()).await?)
  } else {
    None
  };

  let images = LocalImageView::get_all_paged_by_local_user_id(
    &mut context.pool(),
    local_user_view.local_user.id,
    cursor_data,
    data.page_back,
  )
  .await?;

  let next_page = images.last().map(PageCursorBuilder::cursor);

  Ok(Json(ListMediaResponse { images, next_page }))
}
