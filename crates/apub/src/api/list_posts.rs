use crate::{
  api::listing_type_with_default,
  fetcher::resolve_actor_identifier,
  objects::community::ApubCommunity,
};
use activitypub_federation::config::Data;
use actix_web::web::{Json, Query};
use lemmy_api_common::{
  context::LemmyContext,
  post::{GetPosts, GetPostsResponse},
  utils::{check_private_instance, is_mod_or_admin_opt, local_user_view_from_jwt_opt},
};
use lemmy_db_schema::source::{community::Community, local_site::LocalSite};
use lemmy_db_views::post_view::PostQuery;
use lemmy_utils::error::{LemmyError, LemmyErrorExt, LemmyErrorType};

#[tracing::instrument(skip(context))]
pub async fn list_posts(
  data: Query<GetPosts>,
  context: Data<LemmyContext>,
) -> Result<Json<GetPostsResponse>, LemmyError> {
  let local_user_view = local_user_view_from_jwt_opt(data.auth.as_ref(), &context).await;
  let local_site = LocalSite::read(&mut context.pool()).await?;

  check_private_instance(&local_user_view, &local_site)?;

  let sort = data.sort;

  let page = data.page;
  let limit = data.limit;
  let community_id = if let Some(name) = &data.community_name {
    Some(resolve_actor_identifier::<ApubCommunity, Community>(name, &context, &None, true).await?)
      .map(|c| c.id)
  } else {
    data.community_id
  };
  let saved_only = data.saved_only;

  let listing_type = listing_type_with_default(data.type_, &local_site, community_id)?;

  let is_mod_or_admin =
    is_mod_or_admin_opt(&mut context.pool(), local_user_view.as_ref(), community_id)
      .await
      .is_ok();

  let posts = PostQuery::builder()
    .pool(&mut context.pool())
    .local_user(local_user_view.map(|l| l.local_user).as_ref())
    .listing_type(Some(listing_type))
    .sort(sort)
    .community_id(community_id)
    .saved_only(saved_only)
    .page(page)
    .limit(limit)
    .is_mod_or_admin(Some(is_mod_or_admin))
    .build()
    .list()
    .await
    .with_lemmy_type(LemmyErrorType::CouldntGetPosts)?;

  Ok(Json(GetPostsResponse { posts }))
}
