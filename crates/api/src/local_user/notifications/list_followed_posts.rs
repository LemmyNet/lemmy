use actix_web::web::{Data, Json, Query};
use lemmy_api_common::{
  context::LemmyContext,
  person::{GetPosts, GetFollowedCommunityPostsResponse},
  utils::local_user_view_from_jwt,
};
use lemmy_db_views_actor::followed_community_post_view::FollowedCommunityPostQuery;
use lemmy_utils::error::LemmyError;

#[tracing::instrument(skip(context))]
pub async fn list_followed_posts(
  data: Query<GetPosts>,
  context: Data<LemmyContext>,
) -> Result<Json<GetFollowedCommunityPostsResponse>, LemmyError> {
  let local_user_view = local_user_view_from_jwt(&data.auth, &context).await?;

  let sort = data.sort;
  let page = data.page;
  let limit = data.limit;
  let unread_only = data.unread_only.unwrap_or_default();
  let person_id = Some(local_user_view.person.id);

  let posts = FollowedCommunityPostQuery {
    my_person_id: person_id,
    unread_only,
    sort,
    page,
    limit,
  }
  .list(&mut context.pool())
  .await?;

  Ok(Json(GetFollowedCommunityPostsResponse { posts }))
}

