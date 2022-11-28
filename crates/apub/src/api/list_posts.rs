use crate::{
  api::PerformApub,
  fetcher::resolve_actor_identifier,
  objects::community::ApubCommunity,
};
use actix_web::web::Data;
use lemmy_api_common::{
  context::LemmyContext,
  post::{GetPosts, GetPostsResponse},
  utils::{
    check_private_instance,
    get_local_user_view_from_jwt_opt,
    listing_type_with_site_default,
  },
};
use lemmy_db_schema::{
  source::{community::Community, local_site::LocalSite},
  traits::DeleteableOrRemoveable,
};
use lemmy_db_views::post_view::PostQuery;
use lemmy_utils::{error::LemmyError, ConnectionId};

#[async_trait::async_trait(?Send)]
impl PerformApub for GetPosts {
  type Response = GetPostsResponse;

  #[tracing::instrument(skip(context, _websocket_id))]
  async fn perform(
    &self,
    context: &Data<LemmyContext>,
    _websocket_id: Option<ConnectionId>,
  ) -> Result<GetPostsResponse, LemmyError> {
    let data: &GetPosts = self;
    let local_user_view =
      get_local_user_view_from_jwt_opt(data.auth.as_ref(), context.pool(), context.secret())
        .await?;
    let local_site = LocalSite::read(context.pool()).await?;

    check_private_instance(&local_user_view, &local_site)?;

    let is_logged_in = local_user_view.is_some();

    let sort = data.sort;
    let listing_type = listing_type_with_site_default(data.type_, &local_site)?;

    let page = data.page;
    let limit = data.limit;
    let community_id = data.community_id;
    let community_actor_id = if let Some(name) = &data.community_name {
      resolve_actor_identifier::<ApubCommunity, Community>(name, context, true)
        .await
        .ok()
        .map(|c| c.actor_id)
    } else {
      None
    };
    let saved_only = data.saved_only;

    let mut posts = PostQuery::builder()
      .pool(context.pool())
      .local_user(local_user_view.map(|l| l.local_user).as_ref())
      .listing_type(Some(listing_type))
      .sort(sort)
      .community_id(community_id)
      .community_actor_id(community_actor_id)
      .saved_only(saved_only)
      .page(page)
      .limit(limit)
      .build()
      .list()
      .await
      .map_err(|e| LemmyError::from_error_message(e, "couldnt_get_posts"))?;

    // Blank out deleted or removed info for non-logged in users
    if !is_logged_in {
      for pv in posts
        .iter_mut()
        .filter(|p| p.post.deleted || p.post.removed)
      {
        pv.post = pv.clone().post.blank_out_deleted_or_removed_info();
      }

      for pv in posts
        .iter_mut()
        .filter(|p| p.community.deleted || p.community.removed)
      {
        pv.community = pv.clone().community.blank_out_deleted_or_removed_info();
      }
    }

    Ok(GetPostsResponse { posts })
  }
}
