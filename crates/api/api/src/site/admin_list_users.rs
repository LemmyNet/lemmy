use actix_web::web::{Data, Json};
use lemmy_api_utils::{context::LemmyContext, utils::is_admin};
use lemmy_db_schema::traits::PaginationCursorBuilder;
use lemmy_db_views_local_user::{
  api::{AdminListUsers, AdminListUsersResponse},
  impls::LocalUserQuery,
  LocalUserView,
};
use lemmy_db_views_person::PersonView;
use lemmy_utils::error::LemmyResult;

pub async fn admin_list_users(
  data: Json<AdminListUsers>,
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> LemmyResult<Json<AdminListUsersResponse>> {
  // Make sure user is an admin
  is_admin(&local_user_view)?;

  let cursor_data = if let Some(cursor) = &data.page_cursor {
    Some(PersonView::from_cursor(cursor, &mut context.pool()).await?)
  } else {
    None
  };

  let users = LocalUserQuery {
    banned_only: data.banned_only,
    cursor_data,
    page_back: data.page_back,
    limit: data.limit,
  }
  .list(&mut context.pool())
  .await?;

  let next_page = users.last().map(PaginationCursorBuilder::to_cursor);
  let prev_page = users.first().map(PaginationCursorBuilder::to_cursor);

  Ok(Json(AdminListUsersResponse {
    users,
    next_page,
    prev_page,
  }))
}
