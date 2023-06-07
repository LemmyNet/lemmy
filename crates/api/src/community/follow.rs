use crate::Perform;
use actix_web::web::Data;
use lemmy_api_common::{
  community::{CommunityResponse, FollowCommunity},
  context::LemmyContext,
  utils::{check_community_ban, check_community_deleted_or_removed, local_user_view_from_jwt},
};
use lemmy_db_schema::{
  source::{
    actor_language::CommunityLanguage,
    community::{Community, CommunityFollower, CommunityFollowerForm},
  },
  traits::{Crud, Followable},
};
use lemmy_db_views_actor::structs::CommunityView;
use lemmy_utils::error::LemmyError;

#[async_trait::async_trait(?Send)]
impl Perform for FollowCommunity {
  type Response = CommunityResponse;

  #[tracing::instrument(skip(context))]
  async fn perform(&self, context: &Data<LemmyContext>) -> Result<CommunityResponse, LemmyError> {
    let data: &FollowCommunity = self;
    let local_user_view = local_user_view_from_jwt(&data.auth, context).await?;

    let community_id = data.community_id;
    let community = Community::read(context.pool(), community_id).await?;
    let community_follower_form = CommunityFollowerForm {
      community_id: data.community_id,
      person_id: local_user_view.person.id,
      pending: false,
    };

    if community.local && data.follow {
      check_community_ban(local_user_view.person.id, community_id, context.pool()).await?;
      check_community_deleted_or_removed(community_id, context.pool()).await?;

      CommunityFollower::follow(context.pool(), &community_follower_form)
        .await
        .map_err(|e| LemmyError::from_error_message(e, "community_follower_already_exists"))?;
    }
    if !data.follow {
      CommunityFollower::unfollow(context.pool(), &community_follower_form)
        .await
        .map_err(|e| LemmyError::from_error_message(e, "community_follower_already_exists"))?;
    }

    let community_id = data.community_id;
    let person_id = local_user_view.person.id;
    let community_view =
      CommunityView::read(context.pool(), community_id, Some(person_id), None).await?;
    let discussion_languages = CommunityLanguage::read(context.pool(), community_id).await?;

    Ok(Self::Response {
      community_view,
      discussion_languages,
    })
  }
}
