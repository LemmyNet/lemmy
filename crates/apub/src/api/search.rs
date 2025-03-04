use crate::{fetcher::resolve_ap_identifier, objects::community::ApubCommunity};
use activitypub_federation::config::Data;
use actix_web::web::{Json, Query};
use lemmy_api_common::{
  context::LemmyContext,
  site::{Search, SearchResponse},
  utils::{check_conflicting_like_filters, check_private_instance},
};
use lemmy_db_schema::source::community::Community;
use lemmy_db_views::{
  combined::search_combined_view::SearchCombinedQuery,
  structs::{LocalUserView, SiteView},
};
use lemmy_utils::error::LemmyResult;

pub async fn search(
  data: Query<Search>,
  context: Data<LemmyContext>,
  local_user_view: Option<LocalUserView>,
) -> LemmyResult<Json<SearchResponse>> {
  let local_site = SiteView::read_local(&mut context.pool()).await?;

  check_private_instance(&local_user_view, &local_site.local_site)?;
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
  let search_term = data.search_term.clone();
  let time_range_seconds = data.time_range_seconds;

  // parse pagination token
  let page_after = if let Some(pa) = &data.page_cursor {
    Some(pa.read(&mut context.pool()).await?)
  } else {
    None
  };
  let page_back = data.page_back;

  let results = SearchCombinedQuery {
    search_term,
    community_id,
    creator_id: data.creator_id,
    type_: data.type_,
    sort: data.sort,
    time_range_seconds,
    listing_type: data.listing_type,
    title_only: data.title_only,
    post_url_only: data.post_url_only,
    liked_only: data.liked_only,
    disliked_only: data.disliked_only,
    page_after,
    page_back,
  }
  .list(&mut context.pool(), &local_user_view)
  .await?;

  Ok(Json(SearchResponse { results }))
}
