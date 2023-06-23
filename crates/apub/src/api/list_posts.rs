use crate::{
  api::{listing_type_with_default, PerformApub},
  fetcher::resolve_actor_identifier,
  objects::community::ApubCommunity,
};
use activitypub_federation::config::Data;
use lemmy_api_common::{
  context::LemmyContext,
  post::{GetPosts, GetPostsResponse},
  utils::{check_private_instance, local_user_view_from_jwt_opt},
};
use lemmy_db_schema::source::{community::Community, local_site::LocalSite};
use lemmy_db_views::post_view::PostQuery;
use lemmy_utils::error::LemmyError;

#[async_trait::async_trait]
impl PerformApub for GetPosts {
  type Response = GetPostsResponse;

  #[tracing::instrument(skip(context))]
  async fn perform(&self, context: &Data<LemmyContext>) -> Result<GetPostsResponse, LemmyError> {
    let data: &GetPosts = self;
    let local_user_view = local_user_view_from_jwt_opt(data.auth.as_ref(), context).await;
    let local_site = LocalSite::read(context.pool()).await?;

    check_private_instance(&local_user_view, &local_site)?;

    let sort = data.sort;

    let page = data.page;
    let limit = data.limit;
    let community_id = if let Some(name) = &data.community_name {
      resolve_actor_identifier::<ApubCommunity, Community>(name, context, &None, true)
        .await
        .ok()
        .map(|c| c.id)
    } else {
      data.community_id
    };
    let saved_only = data.saved_only;

    let listing_type = listing_type_with_default(data.type_, &local_site, community_id)?;

    let posts = PostQuery::builder()
      .pool(context.pool())
      .local_user(local_user_view.map(|l| l.local_user).as_ref())
      .listing_type(Some(listing_type))
      .sort(sort)
      .community_id(community_id)
      .saved_only(saved_only)
      .page(page)
      .limit(limit)
      .build()
      .list()
      .await
      .map_err(|e| LemmyError::from_error_message(e, "couldnt_get_posts"))?;

    Ok(GetPostsResponse { posts })
  }
}
