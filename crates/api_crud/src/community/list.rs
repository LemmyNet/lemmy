use crate::PerformCrud;
use actix_web::web::Data;
use lemmy_api_common::{
  community::{ListCommunities, ListCommunitiesResponse},
  utils::{check_private_instance, get_local_user_view_from_jwt_opt},
};
use lemmy_db_schema::{source::local_site::LocalSite, traits::DeleteableOrRemoveable};
use lemmy_db_views_actor::community_view::CommunityQuery;
use lemmy_utils::{error::LemmyError, ConnectionId};
use lemmy_websocket::LemmyContext;

#[async_trait::async_trait(?Send)]
impl PerformCrud for ListCommunities {
  type Response = ListCommunitiesResponse;

  #[tracing::instrument(skip(context, _websocket_id))]
  async fn perform(
    &self,
    context: &Data<LemmyContext>,
    _websocket_id: Option<ConnectionId>,
  ) -> Result<ListCommunitiesResponse, LemmyError> {
    let data: &ListCommunities = self;
    let local_user_view =
      get_local_user_view_from_jwt_opt(data.auth.as_ref(), context.pool(), context.secret())
        .await?;
    let local_site = LocalSite::read(context.pool()).await?;

    check_private_instance(&local_user_view, &local_site)?;

    let person_id = local_user_view.clone().map(|l| l.person.id);

    let sort = data.sort;
    let listing_type = data.type_;
    let page = data.page;
    let limit = data.limit;
    let local_user = local_user_view.map(|l| l.local_user);
    let mut communities = CommunityQuery::builder()
      .pool(context.pool())
      .listing_type(listing_type)
      .sort(sort)
      .local_user(local_user.as_ref())
      .page(page)
      .limit(limit)
      .build()
      .list()
      .await?;

    // Blank out deleted or removed info for non-logged in users
    if person_id.is_none() {
      for cv in communities
        .iter_mut()
        .filter(|cv| cv.community.deleted || cv.community.removed)
      {
        cv.community = cv.clone().community.blank_out_deleted_or_removed_info();
      }
    }

    // Return the jwt
    Ok(ListCommunitiesResponse { communities })
  }
}
