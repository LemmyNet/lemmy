use crate::{api::resolve_object::resolve_object_internal, fetcher::resolve_ap_identifier};
use activitypub_federation::config::Data;
use actix_web::web::{Json, Query};
use futures::future::join;
use lemmy_api_utils::{
  context::LemmyContext,
  utils::{check_conflicting_like_filters, check_private_instance},
};
use lemmy_apub_objects::objects::community::ApubCommunity;
use lemmy_db_schema::{source::community::Community, traits::PaginationCursorBuilder};
use lemmy_db_views_local_user::LocalUserView;
use lemmy_db_views_search_combined::{
  impls::SearchCombinedQuery,
  Search,
  SearchCombinedView,
  SearchResponse,
};
use lemmy_db_views_site::SiteView;
use lemmy_utils::error::LemmyResult;

pub async fn search(
  data: Query<Search>,
  context: Data<LemmyContext>,
  local_user_view: Option<LocalUserView>,
) -> LemmyResult<Json<SearchResponse>> {
  let site_view = SiteView::read_local(&mut context.pool()).await?;
  let local_site = site_view.local_site;

  check_private_instance(&local_user_view, &local_site)?;
  check_conflicting_like_filters(data.liked_only, data.disliked_only)?;

  let community_id = if let Some(name) = &data.community_name {
    Some(
      resolve_ap_identifier::<ApubCommunity, Community>(name, &context, &local_user_view, false)
        .await?,
    )
    .map(|c| c.id)
  } else {
    data.community_id
  };

  let cursor_data = if let Some(cursor) = &data.page_cursor {
    Some(SearchCombinedView::from_cursor(cursor, &mut context.pool()).await?)
  } else {
    None
  };

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
    cursor_data,
    page_back: data.page_back,
    limit: data.limit,
  }
  .list(pool, &local_user_view, &site_view.site);

  let resolve_fut = resolve_object_internal(&data.q, &local_user_view, &context);
  let (search, resolve) = join(search_fut, resolve_fut).await;
  let mut search = search?;
  // Ignore resolve errors as query may not be Activitypub object
  if let Ok(resolve) = resolve {
    search.push(resolve);
  }

  let next_page = search.last().map(PaginationCursorBuilder::to_cursor);
  let prev_page = search.first().map(PaginationCursorBuilder::to_cursor);

  Ok(Json(SearchResponse {
    results: search,
    next_page,
    prev_page,
  }))
}
