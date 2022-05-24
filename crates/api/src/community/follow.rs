use crate::Perform;
use actix_web::web::Data;
use lemmy_api_common::{
  community::{FollowCommunity, FollowCommunityResponse},
  utils::{
    blocking,
    check_community_ban,
    check_community_deleted_or_removed,
    get_local_user_view_from_jwt,
  },
};
use lemmy_apub::{
  objects::community::ApubCommunity,
  protocol::activities::following::{
    follow::FollowCommunity as FollowCommunityApub,
    undo_follow::UndoFollowCommunity,
  },
};
use lemmy_db_schema::{
  source::community::{Community, CommunityFollower, CommunityFollowerForm},
  traits::{Crud, Followable},
};
use lemmy_db_views_actor::structs::CommunityFollowerView;
use lemmy_utils::{ConnectionId, LemmyError};
use lemmy_websocket::LemmyContext;

#[async_trait::async_trait(?Send)]
impl Perform for FollowCommunity {
  type Response = FollowCommunityResponse;

  #[tracing::instrument(skip(context, _websocket_id))]
  async fn perform(
    &self,
    context: &Data<LemmyContext>,
    _websocket_id: Option<ConnectionId>,
  ) -> Result<Self::Response, LemmyError> {
    let data: &FollowCommunity = self;
    let local_user_view =
      get_local_user_view_from_jwt(&data.auth, context.pool(), context.secret()).await?;

    let community_id = data.community_id;
    let community: ApubCommunity = blocking(context.pool(), move |conn| {
      Community::read(conn, community_id)
    })
    .await??
    .into();
    let community_follower_form = CommunityFollowerForm {
      community_id: data.community_id,
      person_id: local_user_view.person.id,
      pending: false, // Don't worry, this form isn't used for remote follows
    };

    if community.local {
      if data.follow {
        check_community_ban(local_user_view.person.id, community_id, context.pool()).await?;
        check_community_deleted_or_removed(community_id, context.pool()).await?;

        let follow = move |conn: &'_ _| CommunityFollower::follow(conn, &community_follower_form);
        blocking(context.pool(), follow)
          .await?
          .map_err(|e| LemmyError::from_error_message(e, "community_follower_already_exists"))?;
      } else {
        let unfollow =
          move |conn: &'_ _| CommunityFollower::unfollow(conn, &community_follower_form);
        blocking(context.pool(), unfollow)
          .await?
          .map_err(|e| LemmyError::from_error_message(e, "community_follower_already_exists"))?;
      }
    } else if data.follow {
      // Dont actually add to the community followers here, because you need
      // to wait for the accept
      FollowCommunityApub::send(&local_user_view.person.clone().into(), &community, context)
        .await?;
    } else {
      UndoFollowCommunity::send(&local_user_view.person.clone().into(), &community, context)
        .await?;
      let unfollow = move |conn: &'_ _| CommunityFollower::unfollow(conn, &community_follower_form);
      blocking(context.pool(), unfollow)
        .await?
        .map_err(|e| LemmyError::from_error_message(e, "community_follower_already_exists"))?;
    }

    let community_id = data.community_id;
    let person_id = local_user_view.person.id;
    let community_follower_view = blocking(context.pool(), move |conn| {
      CommunityFollowerView::read(conn, community_id, person_id)
    })
    .await?
    .ok();

    Ok(Self::Response {
      community_follower_view,
    })
  }
}
