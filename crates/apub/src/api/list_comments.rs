use crate::{
  api::{listing_type_with_default, PerformApub},
  fetcher::resolve_actor_identifier,
  objects::community::ApubCommunity,
};
use activitypub_federation::config::Data;
use lemmy_api_common::{
  comment::{GetComments, GetCommentsResponse},
  context::LemmyContext,
  utils::{check_private_instance, local_user_view_from_jwt_opt},
};
use lemmy_db_schema::{
  source::{comment::Comment, community::Community, local_site::LocalSite},
  traits::Crud,
};
use lemmy_db_views::comment_view::CommentQuery;
use lemmy_utils::error::LemmyError;

#[async_trait::async_trait]
impl PerformApub for GetComments {
  type Response = GetCommentsResponse;

  #[tracing::instrument(skip(context))]
  async fn perform(&self, context: &Data<LemmyContext>) -> Result<GetCommentsResponse, LemmyError> {
    let data: &GetComments = self;
    let local_user_view = local_user_view_from_jwt_opt(data.auth.as_ref(), context).await;
    let local_site = LocalSite::read(context.pool()).await?;
    check_private_instance(&local_user_view, &local_site)?;

    let community_id = if let Some(name) = &data.community_name {
      resolve_actor_identifier::<ApubCommunity, Community>(name, context, &None, true)
        .await
        .ok()
        .map(|c| c.id)
    } else {
      data.community_id
    };
    let sort = data.sort;
    let max_depth = data.max_depth;
    let saved_only = data.saved_only;
    let page = data.page;
    let limit = data.limit;
    let parent_id = data.parent_id;

    let listing_type = listing_type_with_default(data.type_, &local_site, community_id)?;

    // If a parent_id is given, fetch the comment to get the path
    let parent_path = if let Some(parent_id) = parent_id {
      Some(Comment::read(context.pool(), parent_id).await?.path)
    } else {
      None
    };

    let parent_path_cloned = parent_path.clone();
    let post_id = data.post_id;
    let local_user = local_user_view.map(|l| l.local_user);
    let comments = CommentQuery::builder()
      .pool(context.pool())
      .listing_type(Some(listing_type))
      .sort(sort)
      .max_depth(max_depth)
      .saved_only(saved_only)
      .community_id(community_id)
      .parent_path(parent_path_cloned)
      .post_id(post_id)
      .local_user(local_user.as_ref())
      .page(page)
      .limit(limit)
      .build()
      .list()
      .await
      .map_err(|e| LemmyError::from_error_message(e, "couldnt_get_comments"))?;

    Ok(GetCommentsResponse { comments })
  }
}
