use actix_web::web::{Data, Json, Query};
use lemmy_api_utils::context::LemmyContext;
use lemmy_db_schema::source::local_user_invite::LocalUserInvite;
use lemmy_db_views_local_user::LocalUserView;
use lemmy_db_views_local_user_invite::{api::ListInvitations, impls::LocalUserInviteQuery};
use lemmy_diesel_utils::pagination::PagedResponse;
use lemmy_utils::error::LemmyResult;

pub async fn list_invitations(
  data: Query<ListInvitations>,
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> LemmyResult<Json<PagedResponse<LocalUserInvite>>> {
  let pool = &mut context.pool();

  let paged = LocalUserInviteQuery {
    local_user_id: local_user_view.local_user.id,
    page_cursor: data.page_cursor.clone(),
    limit: data.limit,
  }
  .list(pool)
  .await?;

  Ok(Json(paged))
}
