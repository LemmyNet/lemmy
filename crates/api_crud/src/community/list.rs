use actix_web::web::{Data, Json, Query};
use lemmy_api_common::{
  community::{ListCommunities, ListCommunitiesResponse},
  context::LemmyContext,
  utils::{check_private_instance, is_admin},
};
use lemmy_db_views::structs::{LocalUserView, SiteView};
use lemmy_db_views_actor::community_view::CommunityQuery;
use lemmy_utils::error::LemmyResult;

#[tracing::instrument(skip(context))]
pub async fn list_communities(
  data: Query<ListCommunities>,
  context: Data<LemmyContext>,
  local_user_view: Option<LocalUserView>,
) -> LemmyResult<Json<ListCommunitiesResponse>> {
  let local_site = SiteView::read_local(&mut context.pool()).await?;
  let is_admin = local_user_view
    .as_ref()
    .map(|luv| is_admin(luv).is_ok())
    .unwrap_or_default();

  check_private_instance(&local_user_view, &local_site.local_site)?;

  let sort = data.sort;
  let listing_type = data.type_;
  let show_nsfw = data.show_nsfw.unwrap_or_default();
  let page = data.page;
  let limit = data.limit;
  let local_user = local_user_view.map(|l| l.local_user);
  let communities = CommunityQuery {
    listing_type,
    show_nsfw,
    sort,
    local_user: local_user.as_ref(),
    page,
    limit,
    is_mod_or_admin: is_admin,
    ..Default::default()
  }
  .list(&local_site.site, &mut context.pool())
  .await?;

  // Return the jwt
  Ok(Json(ListCommunitiesResponse { communities }))
}
