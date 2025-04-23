use activitypub_federation::config::Data;
use actix_web::web::{Json, Query};
use lemmy_api_common::{
  context::LemmyContext,
  person::{ListPersonRead, ListPersonReadResponse},
};
use lemmy_db_schema::source::post::PostActions;
use lemmy_db_views_local_user::LocalUserView;
use lemmy_db_views_post::PostView;
use lemmy_utils::error::LemmyResult;

pub async fn list_person_read(
  data: Query<ListPersonRead>,
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> LemmyResult<Json<ListPersonReadResponse>> {
  let cursor_data = if let Some(cursor) = &data.page_cursor {
    Some(PostActions::from_cursor(cursor, &mut context.pool()).await?)
  } else {
    None
  };

  let read = PostView::list_read(
    &mut context.pool(),
    &local_user_view.person,
    cursor_data,
    data.page_back,
    data.limit,
  )
  .await?;

  let next_page = read.last().map(PostView::to_post_actions_cursor);
  let prev_page = read.first().map(PostView::to_post_actions_cursor);

  Ok(Json(ListPersonReadResponse {
    read,
    next_page,
    prev_page,
  }))
}
