use crate::federation::{
  fetcher::resolve_community_identifier,
  resolve_object::resolve_object_internal,
};
use activitypub_federation::config::Data;
use actix_web::web::{Json, Query};
use futures::future::join;
use lemmy_api_utils::{
  context::LemmyContext,
  utils::{check_conflicting_like_filters, check_private_instance},
};
use lemmy_db_views_local_user::LocalUserView;
use lemmy_db_views_search_combined::{Search, SearchResponse, impls::SearchCombinedQuery};
use lemmy_db_views_site::SiteView;
use lemmy_utils::error::LemmyResult;

pub async fn search(
  Query(data): Query<Search>,
  context: Data<LemmyContext>,
  local_user_view: Option<LocalUserView>,
) -> LemmyResult<Json<SearchResponse>> {
  let site_view = SiteView::read_local(&mut context.pool()).await?;
  let local_site = site_view.local_site;

  check_private_instance(&local_user_view, &local_site)?;
  check_conflicting_like_filters(data.liked_only, data.disliked_only)?;

  let community_id = resolve_community_identifier(
    &data.community_name,
    data.community_id,
    &context,
    &local_user_view,
  )
  .await?;

  let pool = &mut context.pool();
  let search_fut = SearchCombinedQuery {
    search_term: Some(data.q.clone()),
    community_id,
    creator_id: data.creator_id,
    type_: data.type_,
    sort: data.sort,
    time_range_seconds: data.time_range_seconds,
    listing_type: data.listing_type,
    title_only: data.title_only,
    post_url_only: data.post_url_only,
    liked_only: data.liked_only,
    disliked_only: data.disliked_only,
    show_nsfw: data.show_nsfw,
    page_cursor: data.page_cursor,
    limit: data.limit,
  }
  .list(pool, &local_user_view, &site_view.site);

  let resolve_fut = resolve_object_internal(&data.q, &local_user_view, &context);
  let (search, resolve) = join(search_fut, resolve_fut).await;
  let search = search?;

  Ok(Json(SearchResponse {
    search: search.items,
    // ignore errors as this may not be an apub url
    resolve: resolve.ok(),
    next_page: search.next_page,
    prev_page: search.prev_page,
  }))
}
