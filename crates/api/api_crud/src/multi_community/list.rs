use activitypub_federation::config::Data;
use actix_web::web::{Json, Query};
use lemmy_api_utils::context::LemmyContext;
use lemmy_db_schema::traits::PaginationCursorBuilder;
use lemmy_db_views_community::{
  MultiCommunityView,
  api::{ListMultiCommunities, ListMultiCommunitiesResponse},
  impls::MultiCommunityQuery,
};
use lemmy_db_views_local_user::LocalUserView;
use lemmy_utils::error::LemmyResult;

pub async fn list_multi_communities(
  data: Query<ListMultiCommunities>,
  context: Data<LemmyContext>,
  local_user_view: Option<LocalUserView>,
) -> LemmyResult<Json<ListMultiCommunitiesResponse>> {
  let my_person_id = local_user_view.map(|l| l.person.id);

  let cursor_data = if let Some(cursor) = &data.page_cursor {
    Some(MultiCommunityView::from_cursor(cursor, &mut context.pool()).await?)
  } else {
    None
  };

  let multi_communities = MultiCommunityQuery {
    listing_type: data.type_,
    sort: data.sort,
    creator_id: data.creator_id,
    my_person_id,
    time_range_seconds: data.time_range_seconds,
    cursor_data,
    page_back: data.page_back,
    limit: data.limit,
    ..Default::default()
  }
  .list(&mut context.pool())
  .await?;

  let next_page = multi_communities
    .last()
    .map(PaginationCursorBuilder::to_cursor);
  let prev_page = multi_communities
    .first()
    .map(PaginationCursorBuilder::to_cursor);

  Ok(Json(ListMultiCommunitiesResponse {
    multi_communities,
    next_page,
    prev_page,
  }))
}
