use actix_web::web::{Data, Json, Query};
use lemmy_api_common::{
  community::{ListCommunities, ListCommunitiesResponse},
  context::LemmyContext,
  utils::check_private_instance,
};
use lemmy_db_schema::traits::PaginationCursorBuilder;
use lemmy_db_views_community::{impls::CommunityQuery, CommunityView};
use lemmy_db_views_local_user::LocalUserView;
use lemmy_db_views_site::SiteView;
use lemmy_utils::error::LemmyResult;

pub async fn list_communities(
  data: Query<ListCommunities>,
  context: Data<LemmyContext>,
  local_user_view: Option<LocalUserView>,
) -> LemmyResult<Json<ListCommunitiesResponse>> {
  let local_site = SiteView::read_local(&mut context.pool()).await?;

  check_private_instance(&local_user_view, &local_site.local_site)?;

  let local_user = local_user_view.map(|l| l.local_user);

  let cursor_data = if let Some(cursor) = &data.page_cursor {
    Some(CommunityView::from_cursor(cursor, &mut context.pool()).await?)
  } else {
    None
  };

  let communities = CommunityQuery {
    listing_type: data.type_,
    show_nsfw: data.show_nsfw,
    sort: data.sort,
    time_range_seconds: data.time_range_seconds,
    local_user: local_user.as_ref(),
    cursor_data,
    page_back: data.page_back,
    limit: data.limit,
  }
  .list(&local_site.site, &mut context.pool())
  .await?;

  let next_page = communities.last().map(PaginationCursorBuilder::to_cursor);
  let prev_page = communities.first().map(PaginationCursorBuilder::to_cursor);

  // Return the jwt
  Ok(Json(ListCommunitiesResponse {
    communities,
    next_page,
    prev_page,
  }))
}
