use activitypub_federation::config::Data;
use actix_web::web::{Json, Query};
use lemmy_api_utils::{context::LemmyContext, utils::is_mod_or_admin};
use lemmy_db_views_local_user::LocalUserView;
use lemmy_db_views_person::{PersonView, api::ListCommunityFollowers, impls::PersonQuery};
use lemmy_db_views_site::SiteView;
use lemmy_diesel_utils::pagination::PagedResponse;
use lemmy_utils::error::LemmyResult;

pub async fn list_community_followers(
  Query(data): Query<ListCommunityFollowers>,
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> LemmyResult<Json<PagedResponse<PersonView>>> {
  let SiteView { site, .. } = SiteView::read_local(&mut context.pool()).await?;
  is_mod_or_admin(&mut context.pool(), &local_user_view, data.community_id).await?;
  let res = PersonQuery {
    local_user: Some(&local_user_view.local_user),
    sort: data.sort,
    listing_type: data.type_,
    community_id: Some(data.community_id),
    limit: data.limit,
    page_cursor: data.page_cursor,
    ..Default::default()
  }
  .list(&site, &mut context.pool())
  .await?;
  Ok(Json(res))
}
