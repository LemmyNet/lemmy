use crate::{
  api::{
    listing_type_with_default,
    post_sort_type_with_default,
    post_time_range_seconds_with_default,
  },
  fetcher::resolve_ap_identifier,
  objects::community::ApubCommunity,
};
use activitypub_federation::config::Data;
use actix_web::web::{Json, Query};
use lemmy_api_common::{
  context::LemmyContext,
  post::{GetPosts, GetPostsResponse},
  utils::{check_conflicting_like_filters, check_private_instance},
};
use lemmy_db_schema::{
  newtypes::PostId,
  source::{community::Community, keyword_block::LocalUserKeywordBlock, post::PostActions},
  traits::{PaginationCursorBuilder, Readable},
};
use lemmy_db_views_local_user::LocalUserView;
use lemmy_db_views_post::{impls::PostQuery, PostView};
use lemmy_db_views_site::SiteView;
use lemmy_utils::error::LemmyResult;

pub async fn list_posts(
  data: Query<GetPosts>,
  context: Data<LemmyContext>,
  local_user_view: Option<LocalUserView>,
) -> LemmyResult<Json<GetPostsResponse>> {
  let site_view = SiteView::read_local(&mut context.pool()).await?;

  check_private_instance(&local_user_view, &site_view.local_site)?;

  let limit = data.limit;
  let community_id = if let Some(name) = &data.community_name {
    Some(
      resolve_ap_identifier::<ApubCommunity, Community>(name, &context, &local_user_view, true)
        .await?,
    )
    .map(|c| c.id)
  } else {
    data.community_id
  };
  let show_hidden = data.show_hidden;
  let show_read = data.show_read;
  let show_nsfw = data.show_nsfw;
  let hide_media = data.hide_media;
  let no_comments_only = data.no_comments_only;

  let liked_only = data.liked_only;
  let disliked_only = data.disliked_only;
  check_conflicting_like_filters(liked_only, disliked_only)?;

  let local_user = local_user_view.as_ref().map(|u| &u.local_user);
  let listing_type = Some(listing_type_with_default(
    data.type_,
    local_user,
    &site_view.local_site,
    community_id,
  ));

  let sort = Some(post_sort_type_with_default(
    data.sort,
    local_user,
    &site_view.local_site,
  ));
  let time_range_seconds = post_time_range_seconds_with_default(
    data.time_range_seconds,
    local_user,
    &site_view.local_site,
  );
  let keyword_blocks = if let Some(local_user) = local_user {
    Some(LocalUserKeywordBlock::read(&mut context.pool(), local_user.id).await?)
  } else {
    None
  };

  let (cursor_data, cursor_before_data) = if let Some(cursor) = &data.page_cursor {
    (
      Some(PostView::from_cursor(cursor, &mut context.pool()).await?),
      PostView::prefetch_cursor_before_data(
        &mut context.pool(),
        local_user,
        data.page_back,
        data.limit,
      )
      .await?,
    )
  } else {
    (None, None)
  };
  let page_back = data.page_back;

  let posts = PostQuery {
    local_user,
    listing_type,
    sort,
    time_range_seconds,
    community_id,
    liked_only,
    disliked_only,
    limit,
    show_hidden,
    show_read,
    show_nsfw,
    hide_media,
    no_comments_only,
    keyword_blocks,
    cursor_data,
    cursor_before_data,
    page_back,
  }
  .list(&site_view.site, &mut context.pool())
  .await?;

  // If in their user settings (or as part of the API request), auto-mark fetched posts as read
  if let Some(local_user) = local_user {
    if data
      .mark_as_read
      .unwrap_or(local_user.auto_mark_fetched_posts_as_read)
    {
      let post_ids = posts.iter().map(|p| p.post.id).collect::<Vec<PostId>>();
      let forms = PostActions::build_many_read_forms(&post_ids, local_user.person_id);
      PostActions::mark_many_as_read(&mut context.pool(), &forms).await?;
    }
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
