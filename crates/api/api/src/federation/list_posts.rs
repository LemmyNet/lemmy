use crate::federation::{
  fetch_limit_with_default,
  fetcher::{resolve_community_identifier, resolve_multi_community_identifier},
  listing_type_with_default,
  post_sort_type_with_default,
  post_time_range_seconds_with_default,
};
use activitypub_federation::config::Data;
use actix_web::web::{Json, Query};
use lemmy_api_utils::{context::LemmyContext, utils::check_private_instance};
use lemmy_db_schema::{
  newtypes::PostId,
  source::{keyword_block::LocalUserKeywordBlock, post::PostActions},
};
use lemmy_db_views_local_user::LocalUserView;
use lemmy_db_views_post::{PostView, api::GetPosts, impls::PostQuery};
use lemmy_db_views_site::SiteView;
use lemmy_diesel_utils::pagination::PagedResponse;
use lemmy_utils::error::LemmyResult;
use std::cmp::min;

pub async fn list_posts(
  Query(data): Query<GetPosts>,
  context: Data<LemmyContext>,
  local_user_view: Option<LocalUserView>,
) -> LemmyResult<Json<PagedResponse<PostView>>> {
  let SiteView {
    site, local_site, ..
  } = &SiteView::read_local(&mut context.pool()).await?;

  check_private_instance(&local_user_view, local_site)?;

  let community_id = resolve_community_identifier(
    &data.community_name,
    data.community_id,
    &context,
    &local_user_view,
  )
  .await?;

  let multi_community_id = resolve_multi_community_identifier(
    &data.multi_community_name,
    data.multi_community_id,
    &context,
    &local_user_view,
  )
  .await?;

  let GetPosts {
    show_hidden,
    show_read,
    // Show nsfw content if param is true, or if content_warning exists
    show_nsfw,
    hide_media,
    no_comments_only,
    search_term,
    search_title_only,
    search_url_only,
    page_cursor,
    ..
  } = data;

  let local_user = local_user_view.as_ref().map(|u| &u.local_user);
  let listing_type = Some(listing_type_with_default(
    data.type_,
    local_user,
    local_site,
    community_id,
  ));

  let sort = Some(post_sort_type_with_default(
    data.sort, local_user, local_site,
  ));
  let time_range_seconds =
    post_time_range_seconds_with_default(data.time_range_seconds, local_user, local_site);
  let limit = Some(fetch_limit_with_default(data.limit, local_user, local_site));

  let keyword_blocks = if let Some(local_user) = local_user {
    Some(LocalUserKeywordBlock::read(&mut context.pool(), local_user.id).await?)
  } else {
    None
  };
  // dont allow more than page 10 for performance reasons
  let page = data.page.map(|p| min(p, 10));

  let posts = PostQuery {
    local_user,
    listing_type,
    sort,
    time_range_seconds,
    community_id,
    multi_community_id,
    page,
    limit,
    show_hidden,
    show_read,
    show_nsfw,
    hide_media,
    no_comments_only,
    keyword_blocks,
    search_term,
    search_title_only,
    search_url_only,
    page_cursor,
  }
  .list(&mut context.pool(), site, local_site)
  .await?;

  // If in their user settings (or as part of the API request), auto-mark fetched posts as read
  if let Some(local_user) = local_user
    && data
      .mark_as_read
      .unwrap_or(local_user.auto_mark_fetched_posts_as_read)
  {
    let post_ids = posts.iter().map(|p| p.post.id).collect::<Vec<PostId>>();
    PostActions::mark_as_read(&mut context.pool(), local_user.person_id, &post_ids).await?;
  }

  Ok(Json(posts))
}
