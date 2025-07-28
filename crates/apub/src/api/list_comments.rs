use super::comment_sort_type_with_default;
use crate::{
  api::{
    fetch_limit_with_default,
    listing_type_with_default,
    post_time_range_seconds_with_default,
  },
  fetcher::resolve_ap_identifier,
};
use activitypub_federation::config::Data;
use actix_web::web::{Json, Query};
use lemmy_api_utils::{context::LemmyContext, utils::check_private_instance};
use lemmy_apub_objects::objects::community::ApubCommunity;
use lemmy_db_schema::{
  newtypes::PaginationCursor,
  source::{comment::Comment, community::Community},
  traits::{Crud, PaginationCursorBuilder},
};
use lemmy_db_views_comment::{
  api::{GetComments, GetCommentsResponse, GetCommentsSlimResponse},
  impls::CommentQuery,
  CommentView,
};
use lemmy_db_views_local_user::LocalUserView;
use lemmy_db_views_site::SiteView;
use lemmy_utils::error::LemmyResult;

struct CommentsCommonOutput {
  comments: Vec<CommentView>,
  next_page: Option<PaginationCursor>,
  prev_page: Option<PaginationCursor>,
}

/// A common fetcher for both the CommentView, and CommentSlimView.
async fn list_comments_common(
  data: Query<GetComments>,
  context: Data<LemmyContext>,
  local_user_view: Option<LocalUserView>,
) -> LemmyResult<CommentsCommonOutput> {
  let site_view = SiteView::read_local(&mut context.pool()).await?;
  let local_site = &site_view.local_site;

  check_private_instance(&local_user_view, local_site)?;

  let community_id = if let Some(name) = &data.community_name {
    Some(
      resolve_ap_identifier::<ApubCommunity, Community>(name, &context, &local_user_view, true)
        .await?,
    )
    .map(|c| c.id)
  } else {
    data.community_id
  };
  let local_user = local_user_view.as_ref().map(|u| &u.local_user);
  let sort = Some(comment_sort_type_with_default(
    data.sort, local_user, local_site,
  ));
  let time_range_seconds =
    post_time_range_seconds_with_default(data.time_range_seconds, local_user, local_site);
  let limit = Some(fetch_limit_with_default(data.limit, local_user, local_site));
  let max_depth = data.max_depth;
  let parent_id = data.parent_id;

  let listing_type = Some(listing_type_with_default(
    data.type_,
    local_user_view.as_ref().map(|u| &u.local_user),
    local_site,
    community_id,
  ));

  // If a parent_id is given, fetch the comment to get the path
  let parent_path_ = if let Some(parent_id) = parent_id {
    Some(Comment::read(&mut context.pool(), parent_id).await?.path)
  } else {
    None
  };

  let parent_path = parent_path_.clone();
  let post_id = data.post_id;
  let local_user = local_user_view.as_ref().map(|l| &l.local_user);

  let cursor_data = if let Some(cursor) = &data.page_cursor {
    Some(CommentView::from_cursor(cursor, &mut context.pool()).await?)
  } else {
    None
  };
  let page_back = data.page_back;

  let comments = CommentQuery {
    listing_type,
    sort,
    time_range_seconds,
    max_depth,
    community_id,
    parent_path,
    post_id,
    local_user,
    cursor_data,
    page_back,
    limit,
  }
  .list(&site_view.site, &mut context.pool())
  .await?;

  let next_page = comments.last().map(PaginationCursorBuilder::to_cursor);
  let prev_page = comments.first().map(PaginationCursorBuilder::to_cursor);

  Ok(CommentsCommonOutput {
    comments,
    next_page,
    prev_page,
  })
}

pub async fn list_comments(
  data: Query<GetComments>,
  context: Data<LemmyContext>,
  local_user_view: Option<LocalUserView>,
) -> LemmyResult<Json<GetCommentsResponse>> {
  let common = list_comments_common(data, context, local_user_view).await?;

  Ok(Json(GetCommentsResponse {
    comments: common.comments,
    next_page: common.next_page,
    prev_page: common.prev_page,
  }))
}

pub async fn list_comments_slim(
  data: Query<GetComments>,
  context: Data<LemmyContext>,
  local_user_view: Option<LocalUserView>,
) -> LemmyResult<Json<GetCommentsSlimResponse>> {
  let common = list_comments_common(data, context, local_user_view).await?;

  let comments = common
    .comments
    .into_iter()
    .map(CommentView::map_to_slim)
    .collect();

  Ok(Json(GetCommentsSlimResponse {
    comments,
    next_page: common.next_page,
    prev_page: common.prev_page,
  }))
}
