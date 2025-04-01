use super::comment_sort_type_with_default;
use crate::{
  api::listing_type_with_default,
  fetcher::resolve_ap_identifier,
  objects::community::ApubCommunity,
};
use activitypub_federation::config::Data;
use actix_web::web::{Json, Query};
use lemmy_api_common::{
  comment::{GetComments, GetCommentsResponse, GetCommentsSlimResponse},
  context::LemmyContext,
  utils::{check_conflicting_like_filters, check_private_instance},
};
use lemmy_db_schema::{
  source::{comment::Comment, community::Community},
  traits::Crud,
};
use lemmy_db_views::{
  comment::comment_view::CommentQuery,
  structs::{CommentView, LocalUserView, SiteView},
};
use lemmy_utils::error::{LemmyErrorExt, LemmyErrorType, LemmyResult};

/// A common fetcher for both the CommentView, and CommentSlimView.
async fn list_comments_common(
  data: Query<GetComments>,
  context: Data<LemmyContext>,
  local_user_view: Option<LocalUserView>,
) -> LemmyResult<Vec<CommentView>> {
  let site_view = SiteView::read_local(&mut context.pool()).await?;
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
  let local_user_ref = local_user_view.as_ref().map(|u| &u.local_user);
  let sort = Some(comment_sort_type_with_default(
    data.sort,
    local_user_ref,
    &site_view.local_site,
  ));
  let time_range_seconds = data.time_range_seconds;
  let max_depth = data.max_depth;

  let liked_only = data.liked_only;
  let disliked_only = data.disliked_only;
  check_conflicting_like_filters(liked_only, disliked_only)?;

  let page = data.page;
  let limit = data.limit;
  let parent_id = data.parent_id;

  let listing_type = Some(listing_type_with_default(
    data.type_,
    local_user_view.as_ref().map(|u| &u.local_user),
    &site_view.local_site,
    community_id,
  ));

  // If a parent_id is given, fetch the comment to get the path
  let parent_path = if let Some(parent_id) = parent_id {
    Some(Comment::read(&mut context.pool(), parent_id).await?.path)
  } else {
    None
  };

  let parent_path_cloned = parent_path.clone();
  let post_id = data.post_id;
  let local_user = local_user_view.as_ref().map(|l| &l.local_user);

  CommentQuery {
    listing_type,
    sort,
    time_range_seconds,
    max_depth,
    liked_only,
    disliked_only,
    community_id,
    parent_path: parent_path_cloned,
    post_id,
    local_user,
    page,
    limit,
    ..Default::default()
  }
  .list(&site_view.site, &mut context.pool())
  .await
  .with_lemmy_type(LemmyErrorType::CouldntGetComments)
}

pub async fn list_comments(
  data: Query<GetComments>,
  context: Data<LemmyContext>,
  local_user_view: Option<LocalUserView>,
) -> LemmyResult<Json<GetCommentsResponse>> {
  let comments = list_comments_common(data, context, local_user_view).await?;

  Ok(Json(GetCommentsResponse { comments }))
}

pub async fn list_comments_slim(
  data: Query<GetComments>,
  context: Data<LemmyContext>,
  local_user_view: Option<LocalUserView>,
) -> LemmyResult<Json<GetCommentsSlimResponse>> {
  let comments = list_comments_common(data, context, local_user_view)
    .await?
    .into_iter()
    .map(CommentView::map_to_slim)
    .collect();

  Ok(Json(GetCommentsSlimResponse { comments }))
}
