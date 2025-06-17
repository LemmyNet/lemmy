use activitypub_federation::config::Data;
use actix_web::web::{Json, Query};
use lemmy_api_utils::context::LemmyContext;
use lemmy_db_schema::source::post::PostActions;
use lemmy_db_views_local_user::LocalUserView;
use lemmy_db_views_post::PostView;
use lemmy_db_views_site::api::{ListPersonHidden, ListPersonHiddenResponse};
use lemmy_utils::error::LemmyResult;

pub async fn list_person_hidden(
  data: Query<ListPersonHidden>,
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> LemmyResult<Json<ListPersonHiddenResponse>> {
  let cursor_data = if let Some(cursor) = &data.page_cursor {
    Some(PostActions::from_cursor(cursor, &mut context.pool()).await?)
  } else {
    None
  };

  let hidden = PostView::list_hidden(
    &mut context.pool(),
    &local_user_view.person,
    cursor_data,
    data.page_back,
    data.limit,
    None,
  )
  .await?;

  let next_page = hidden.last().map(PostView::to_post_actions_cursor);
  let prev_page = hidden.first().map(PostView::to_post_actions_cursor);

  Ok(Json(ListPersonHiddenResponse {
    hidden,
    next_page,
    prev_page,
  }))
}
