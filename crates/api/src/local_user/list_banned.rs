use actix_web::web::{Data, Json};
use lemmy_api_common::{
  context::LemmyContext,
  person::{BannedPersonsResponse, ListBannedPersons},
  utils::is_admin,
};
use lemmy_db_schema::traits::PaginationCursorBuilder;
use lemmy_db_views_local_user::LocalUserView;
use lemmy_db_views_person::{impls::PersonQuery, PersonView};
use lemmy_utils::error::LemmyResult;

pub async fn list_banned_users(
  data: Json<ListBannedPersons>,
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> LemmyResult<Json<BannedPersonsResponse>> {
  // Make sure user is an admin
  is_admin(&local_user_view)?;

  let cursor_data = if let Some(cursor) = &data.page_cursor {
    Some(PersonView::from_cursor(cursor, &mut context.pool()).await?)
  } else {
    None
  };

  let banned = PersonQuery {
    banned_only: Some(true),
    cursor_data,
    limit: data.limit,
    page_back: data.page_back,
    ..Default::default()
  }
  .list(local_user_view.person.instance_id, &mut context.pool())
  .await?;

  let next_page = banned.last().map(PaginationCursorBuilder::to_cursor);
  let prev_page = banned.first().map(PaginationCursorBuilder::to_cursor);

  Ok(Json(BannedPersonsResponse {
    banned,
    next_page,
    prev_page,
  }))
}
