use actix_web::web::{Data, Json, Query};
use lemmy_api_utils::{context::LemmyContext, utils::check_private_instance};
use lemmy_db_views_community::{
  api::{ListCommunities, ListCommunitiesResponse},
  impls::CommunityQuery,
};
use lemmy_db_views_local_user::LocalUserView;
use lemmy_db_views_site::SiteView;
use lemmy_utils::error::LemmyResult;

pub async fn list_communities(
  Query(data): Query<ListCommunities>,
  context: Data<LemmyContext>,
  local_user_view: Option<LocalUserView>,
) -> LemmyResult<Json<ListCommunitiesResponse>> {
  let local_site = SiteView::read_local(&mut context.pool()).await?;

  check_private_instance(&local_user_view, &local_site.local_site)?;

  let local_user = local_user_view.map(|l| l.local_user);

  // Show nsfw content if param is true, or if content_warning exists
  let show_nsfw = data
    .show_nsfw
    .unwrap_or(local_site.site.content_warning.is_some());

  let res = CommunityQuery {
    listing_type: data.type_,
    show_nsfw: Some(show_nsfw),
    sort: data.sort,
    time_range_seconds: data.time_range_seconds,
    local_user: local_user.as_ref(),
    page_cursor: data.page_cursor,
    limit: data.limit,
    ..Default::default()
  }
  .list(&local_site.site, &mut context.pool())
  .await?;

  // Return the jwt
  Ok(Json(ListCommunitiesResponse {
    communities: res.data,
    next_page: res.next_page,
    prev_page: res.prev_page,
  }))
}
