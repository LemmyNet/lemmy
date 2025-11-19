use activitypub_federation::config::Data;
use actix_web::web::{Json, Query};
use lemmy_api_utils::context::LemmyContext;
use lemmy_db_views_community::{
  MultiCommunityView,
  api::ListMultiCommunities,
  impls::MultiCommunityQuery,
};
use lemmy_db_views_local_user::LocalUserView;
use lemmy_diesel_utils::pagination::PagedResponse;
use lemmy_utils::error::LemmyResult;

pub async fn list_multi_communities(
  Query(data): Query<ListMultiCommunities>,
  context: Data<LemmyContext>,
  local_user_view: Option<LocalUserView>,
) -> LemmyResult<Json<PagedResponse<MultiCommunityView>>> {
  let my_person_id = local_user_view.map(|l| l.person.id);

  let res = MultiCommunityQuery {
    listing_type: data.type_,
    sort: data.sort,
    creator_id: data.creator_id,
    my_person_id,
    time_range_seconds: data.time_range_seconds,
    page_cursor: data.page_cursor,
    limit: data.limit,
    ..Default::default()
  }
  .list(&mut context.pool())
  .await?;

  Ok(Json(res))
}
