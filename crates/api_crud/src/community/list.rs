use crate::PerformCrud;
use actix_web::web::Data;
use lemmy_api_common::{
  community::{ListCommunities, ListCommunitiesResponse},
  context::LemmyContext,
  utils::{check_private_instance, is_admin, local_user_view_from_jwt_opt},
};
use lemmy_db_schema::source::local_site::LocalSite;
use lemmy_db_views_actor::community_view::CommunityQuery;
use lemmy_utils::error::LemmyError;

#[async_trait::async_trait(?Send)]
impl PerformCrud for ListCommunities {
  type Response = ListCommunitiesResponse;

  #[tracing::instrument(skip(context))]
  async fn perform(
    &self,
    context: &Data<LemmyContext>,
  ) -> Result<ListCommunitiesResponse, LemmyError> {
    let data: &ListCommunities = self;
    let local_user_view = local_user_view_from_jwt_opt(data.auth.as_ref(), context).await;
    let local_site = LocalSite::read(&mut context.pool()).await?;
    let is_admin = local_user_view.as_ref().map(|luv| is_admin(luv).is_ok());

    check_private_instance(&local_user_view, &local_site)?;

    let sort = data.sort;
    let listing_type = data.type_;
    let show_nsfw = data.show_nsfw;
    let page = data.page;
    let limit = data.limit;
    let local_user = local_user_view.map(|l| l.local_user);
    let communities = CommunityQuery::builder()
      .pool(&mut context.pool())
      .listing_type(listing_type)
      .show_nsfw(show_nsfw)
      .sort(sort)
      .local_user(local_user.as_ref())
      .page(page)
      .limit(limit)
      .is_mod_or_admin(is_admin)
      .build()
      .list()
      .await?;

    // Return the jwt
    Ok(ListCommunitiesResponse { communities })
  }
}
