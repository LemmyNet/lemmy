use crate::federation::{
  fetch_limit_with_default,
  fetcher::resolve_ap_identifier,
  listing_type_with_default,
  post_sort_type_with_default,
  post_time_range_seconds_with_default,
};
use activitypub_federation::config::Data;
use actix_web::web::{Json, Query};
use lemmy_api_utils::{context::LemmyContext, utils::check_private_instance};
use lemmy_apub_objects::objects::community::ApubCommunity;
use lemmy_db_schema::{
  newtypes::PostId,
  source::{community::Community, keyword_block::LocalUserKeywordBlock, post::PostActions},
  traits::PaginationCursorBuilder,
};
use lemmy_db_views_local_user::LocalUserView;
use lemmy_db_views_post::{
  PostView,
  api::{GetPosts, GetPostsResponse},
  impls::PostQuery,
};
use lemmy_db_views_site::SiteView;
use lemmy_utils::error::LemmyResult;

pub async fn list_posts(
  data: Query<GetPosts>,
  context: Data<LemmyContext>,
  local_user_view: Option<LocalUserView>,
) -> LemmyResult<Json<GetPostsResponse>> {
  let site_view = SiteView::read_local(&mut context.pool()).await?;
  let local_site = &site_view.local_site;

  check_private_instance(&local_user_view, &site_view.local_site)?;

  let community_id = if let Some(name) = &data.community_name {
    Some(
      resolve_ap_identifier::<ApubCommunity, Community>(name, &context, &local_user_view, true)
        .await?,
    )
    .map(|c| c.id)
  } else {
    data.community_id
  };
  let multi_community_id = data.multi_community_id;
  let show_hidden = data.show_hidden;
  let show_read = data.show_read;
  // Show nsfw content if param is true, or if content_warning exists
  let show_nsfw = data.show_nsfw;
  let hide_media = data.hide_media;
  let no_comments_only = data.no_comments_only;

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

  let cursor_data = if let Some(cursor) = &data.page_cursor {
    Some(PostView::from_cursor(cursor, &mut context.pool()).await?)
  } else {
    None
  };
  let page_back = data.page_back;

  let posts = PostQuery {
    local_user,
    listing_type,
    sort,
    time_range_seconds,
    community_id,
    multi_community_id,
    limit,
    show_hidden,
    show_read,
    show_nsfw,
    hide_media,
    no_comments_only,
    keyword_blocks,
    cursor_data,
    page_back,
  }
  .list(&site_view.site, &mut context.pool())
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

  // if this page wasn't empty, then there is a next page after the last post on this page
  let next_page = posts.last().map(PaginationCursorBuilder::to_cursor);
  let prev_page = posts.first().map(PaginationCursorBuilder::to_cursor);

  Ok(Json(GetPostsResponse {
    posts,
    next_page,
    prev_page,
  }))
}
