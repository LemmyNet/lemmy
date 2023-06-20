use crate::Perform;
use actix_web::web::Data;
use lemmy_api_common::{
  community::{MuteCommunity, MuteCommunityResponse},
  context::LemmyContext,
  utils::local_user_view_from_jwt,
};
use lemmy_db_schema::{
  source::{
    community::{CommunityFollower, CommunityFollowerForm},
    community_mute::{CommunityMute, CommunityMuteForm},
  },
  traits::{Followable, Muteable},
};
use lemmy_db_views_actor::structs::CommunityView;
use lemmy_utils::error::LemmyError;

#[async_trait::async_trait(?Send)]
impl Perform for MuteCommunity {
  type Response = MuteCommunityResponse;

  #[tracing::instrument(skip(context))]
  async fn perform(
    &self,
    context: &Data<LemmyContext>,
  ) -> Result<MuteCommunityResponse, LemmyError> {
    let data: &MuteCommunity = self;
    let local_user_view = local_user_view_from_jwt(&data.auth, context).await?;

    let community_id = data.community_id;
    let person_id = local_user_view.person.id;
    let community_mute_form = CommunityMuteForm {
      person_id,
      community_id,
    };

    if data.mute {
      CommunityMute::mute(context.pool(), &community_mute_form)
        .await
        .map_err(|e| LemmyError::from_error_message(e, "community_mute_already_exists"))?;

      // Also, unfollow the community, and send a federated unfollow
      let community_follower_form = CommunityFollowerForm {
        community_id: data.community_id,
        person_id,
        pending: false,
      };

      CommunityFollower::unfollow(context.pool(), &community_follower_form)
        .await
        .ok();
    } else {
      CommunityMute::unmute(context.pool(), &community_mute_form)
        .await
        .map_err(|e| LemmyError::from_error_message(e, "community_mute_already_exists"))?;
    }

    let community_view =
      CommunityView::read(context.pool(), community_id, Some(person_id), None).await?;

    Ok(MuteCommunityResponse {
      muted: data.mute,
      community_view,
    })
  }
}
