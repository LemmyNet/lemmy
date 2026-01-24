use actix_web::web::{Data, Json, Query};
use lemmy_api_utils::{context::LemmyContext, utils::is_admin};
use lemmy_db_views_local_user::{LocalUserView, api::AdminListUsers, impls::LocalUserQuery};
use lemmy_diesel_utils::pagination::PagedResponse;
use lemmy_utils::error::LemmyResult;

pub async fn admin_list_users(
  Query(data): Query<AdminListUsers>,
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> LemmyResult<Json<PagedResponse<LocalUserView>>> {
  // Make sure user is an admin
  is_admin(&local_user_view)?;

  let users = LocalUserQuery {
    banned_only: data.banned_only,
    page_cursor: data.page_cursor,
    limit: data.limit,
    sort: data.sort,
  }
  .list(&mut context.pool())
  .await?;

  Ok(Json(users))
}
