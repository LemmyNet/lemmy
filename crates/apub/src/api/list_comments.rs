use crate::{
  api::listing_type_with_default,
  fetcher::resolve_actor_identifier,
  objects::community::ApubCommunity,
};
use activitypub_federation::config::Data;
use actix_web::web::{Json, Query};
use lemmy_api_common::{
  comment::{GetComments, GetCommentsResponse},
  context::LemmyContext,
  utils::check_private_instance,
};
use lemmy_db_schema::{
  source::{comment::Comment, community::Community, local_site::LocalSite},
  traits::Crud,
};
use lemmy_db_views::{comment_view::CommentQuery, structs::LocalUserView};
use lemmy_utils::error::{LemmyError, LemmyErrorExt, LemmyErrorType, LemmyResult};

#[tracing::instrument(skip(context))]
pub async fn list_comments(
  data: Query<GetComments>,
  context: Data<LemmyContext>,
  local_user_view: Option<LocalUserView>,
) -> LemmyResult<Json<GetCommentsResponse>> {
  let local_site = LocalSite::read(&mut context.pool()).await?;
  check_private_instance(&local_user_view, &local_site)?;

  let community_id = if let Some(name) = &data.community_name {
    Some(
      resolve_actor_identifier::<ApubCommunity, Community>(name, &context, &local_user_view, true)
        .await?,
    )
    .map(|c| c.id)
  } else {
    data.community_id
  };
  let sort = data.sort;
  let max_depth = data.max_depth;
  let saved_only = data.saved_only.unwrap_or_default();

  let liked_only = data.liked_only.unwrap_or_default();
  let disliked_only = data.disliked_only.unwrap_or_default();
  if liked_only && disliked_only {
    return Err(LemmyError::from(LemmyErrorType::ContradictingFilters));
  }

  let page = data.page;
  let limit = data.limit;
  let parent_id = data.parent_id;

  let listing_type = Some(listing_type_with_default(
    data.type_,
    local_user_view.as_ref().map(|u| &u.local_user),
    &local_site,
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
  let comments = CommentQuery {
    listing_type,
    sort,
    max_depth,
    saved_only,
    liked_only,
    disliked_only,
    community_id,
    parent_path: parent_path_cloned,
    post_id,
    local_user: local_user_view.as_ref(),
    page,
    limit,
    ..Default::default()
  }
  .list(&mut context.pool())
  .await
  .with_lemmy_type(LemmyErrorType::CouldntGetComments)?;

  Ok(Json(GetCommentsResponse { comments }))
}
