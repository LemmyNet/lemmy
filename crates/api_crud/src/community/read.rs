use crate::PerformCrud;
use actix_web::web::Data;
use lemmy_api_common::{blocking, community::*, get_local_user_view_from_jwt_opt};
use lemmy_db_queries::{source::community::Community_, ListingType, SortType};
use lemmy_db_schema::source::community::*;
use lemmy_db_views_actor::{
  community_moderator_view::CommunityModeratorView,
  community_view::{CommunityQueryBuilder, CommunityView},
};
use lemmy_utils::{ApiError, ConnectionId, LemmyError};
use lemmy_websocket::{messages::GetCommunityUsersOnline, LemmyContext};
use std::str::FromStr;

#[async_trait::async_trait(?Send)]
impl PerformCrud for GetCommunity {
  type Response = GetCommunityResponse;

  async fn perform(
    &self,
    context: &Data<LemmyContext>,
    _websocket_id: Option<ConnectionId>,
  ) -> Result<GetCommunityResponse, LemmyError> {
    let data: &GetCommunity = &self;
    let local_user_view = get_local_user_view_from_jwt_opt(&data.auth, context.pool()).await?;
    let person_id = local_user_view.map(|u| u.person.id);

    let community_id = match data.id {
      Some(id) => id,
      None => {
        let name = data.name.to_owned().unwrap_or_else(|| "main".to_string());
        match blocking(context.pool(), move |conn| {
          Community::read_from_name(conn, &name)
        })
        .await?
        {
          Ok(community) => community,
          Err(_e) => return Err(ApiError::err("couldnt_find_community").into()),
        }
        .id
      }
    };

    let community_view = match blocking(context.pool(), move |conn| {
      CommunityView::read(conn, community_id, person_id)
    })
    .await?
    {
      Ok(community) => community,
      Err(_e) => return Err(ApiError::err("couldnt_find_community").into()),
    };

    let moderators: Vec<CommunityModeratorView> = match blocking(context.pool(), move |conn| {
      CommunityModeratorView::for_community(conn, community_id)
    })
    .await?
    {
      Ok(moderators) => moderators,
      Err(_e) => return Err(ApiError::err("couldnt_find_community").into()),
    };

    let online = context
      .chat_server()
      .send(GetCommunityUsersOnline { community_id })
      .await
      .unwrap_or(1);

    let res = GetCommunityResponse {
      community_view,
      moderators,
      online,
    };

    // Return the jwt
    Ok(res)
  }
}

#[async_trait::async_trait(?Send)]
impl PerformCrud for ListCommunities {
  type Response = ListCommunitiesResponse;

  async fn perform(
    &self,
    context: &Data<LemmyContext>,
    _websocket_id: Option<ConnectionId>,
  ) -> Result<ListCommunitiesResponse, LemmyError> {
    let data: &ListCommunities = &self;
    let local_user_view = get_local_user_view_from_jwt_opt(&data.auth, context.pool()).await?;

    let person_id = local_user_view.to_owned().map(|l| l.person.id);

    // Don't show NSFW by default
    let show_nsfw = match &local_user_view {
      Some(uv) => uv.local_user.show_nsfw,
      None => false,
    };

    let type_ = ListingType::from_str(&data.type_)?;
    let sort = SortType::from_str(&data.sort)?;

    let page = data.page;
    let limit = data.limit;
    let communities = blocking(context.pool(), move |conn| {
      CommunityQueryBuilder::create(conn)
        .listing_type(&type_)
        .sort(&sort)
        .show_nsfw(show_nsfw)
        .my_person_id(person_id)
        .page(page)
        .limit(limit)
        .list()
    })
    .await??;

    // Return the jwt
    Ok(ListCommunitiesResponse { communities })
  }
}
