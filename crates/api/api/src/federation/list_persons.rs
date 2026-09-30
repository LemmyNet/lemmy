use activitypub_federation::config::Data;
use actix_web::web::{Json, Query};
use lemmy_api_utils::{
  context::LemmyContext,
  utils::{check_private_instance, is_mod_or_admin_opt},
};
use lemmy_db_views_local_user::LocalUserView;
use lemmy_db_views_person::{PersonView, api::ListPersons, impls::PersonQuery};
use lemmy_db_views_site::SiteView;
use lemmy_diesel_utils::pagination::PagedResponse;
use lemmy_utils::error::LemmyResult;

pub async fn list_persons(
  Query(data): Query<ListPersons>,
  context: Data<LemmyContext>,
  local_user_view: Option<LocalUserView>,
) -> LemmyResult<Json<PagedResponse<PersonView>>> {
  let SiteView {
    site, local_site, ..
  } = SiteView::read_local(&mut context.pool()).await?;

  check_private_instance(&local_user_view, &local_site)?;

  // if community_id is some, returns community subscribers.
  // Works only for community mods and admins

  if data.community_id.is_some() {
    is_mod_or_admin_opt(
      &mut context.pool(),
      local_user_view.as_ref(),
      data.community_id,
    )
    .await?;
  }

  let res = PersonQuery {
    local_user: local_user_view.map(|l| l.local_user).as_ref(),
    sort: data.sort,
    listing_type: data.type_,
    search_term: data.search_term,
    search_title_only: data.search_title_only,
    community_id: data.community_id,
    limit: data.limit,
    page_cursor: data.page_cursor,
  }
  .list(&site, &mut context.pool())
  .await?;

  Ok(Json(res))
}
