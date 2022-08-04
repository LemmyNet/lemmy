use crate::PerformCrud;
use actix_web::web::Data;
use lemmy_api_common::{
  community::{ListCommunities, ListCommunitiesResponse},
  utils::{blocking, check_private_instance, get_local_user_view_from_jwt_opt},
};
use lemmy_db_schema::traits::DeleteableOrRemoveable;
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

    check_private_instance(&local_user_view, context.pool()).await?;

    let person_id = local_user_view.to_owned().map(|l| l.person.id);

    // Don't show NSFW by default
    let show_nsfw = match &local_user_view {
      Some(uv) => uv.local_user.show_nsfw,
      None => false,
    };

    let sort = data.sort;
    let listing_type = data.type_;
    let page = data.page;
    let limit = data.limit;
    let mut communities = blocking(context.pool(), move |conn| {
      CommunityQuery::builder()
        .conn(conn)
        .listing_type(listing_type)
        .sort(sort)
        .show_nsfw(Some(show_nsfw))
        .my_person_id(person_id)
        .page(page)
        .limit(limit)
        .build()
        .list()
    })
    .await??;

    // Blank out deleted or removed info for non-logged in users
    if person_id.is_none() {
      for cv in communities
        .iter_mut()
        .filter(|cv| cv.community.deleted || cv.community.removed)
      {
        cv.community = cv.to_owned().community.blank_out_deleted_or_removed_info();
      }
    }

    // Return the jwt
    Ok(ListCommunitiesResponse { communities })
  }
}
