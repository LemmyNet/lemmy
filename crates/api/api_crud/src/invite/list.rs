use actix_web::web::{Data, Json, Query};
use lemmy_api_utils::{context::LemmyContext, utils::is_admin};
use lemmy_db_schema::InvitationListingType;
use lemmy_db_views_local_user::LocalUserView;
use lemmy_db_views_local_user_invite::{
  api::{ListInvitations, LocalUserInviteView},
  impls::LocalUserInviteQuery,
};
use lemmy_diesel_utils::pagination::PagedResponse;
use lemmy_utils::error::LemmyResult;

pub async fn list_invitations(
  data: Query<ListInvitations>,
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> LemmyResult<Json<PagedResponse<LocalUserInviteView>>> {
  if let Some(InvitationListingType::All) = data.type_ {
    is_admin(&local_user_view)?
  }

  let pool = &mut context.pool();
  let settings = context.settings();

  let paged = LocalUserInviteQuery {
    local_user_id: Some(local_user_view.local_user.id),
    listing_type: data.type_,
    status: data.status,
    page_cursor: data.page_cursor.clone(),
    limit: data.limit,
  }
  .list(pool)
  .await?;

  let items = paged
    .items
    .into_iter()
    .map(|invite| {
      let invite_link = invite.get_invite_url(settings)?;
      Ok(LocalUserInviteView {
        invite,
        invite_link,
      })
    })
    .collect::<LemmyResult<Vec<LocalUserInviteView>>>()?;

  Ok(Json(PagedResponse {
    items,
    next_page: paged.next_page,
    prev_page: paged.prev_page,
  }))
}
