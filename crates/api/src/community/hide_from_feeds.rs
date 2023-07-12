use crate::Perform;
use actix_web::web::Data;
use lemmy_api_common::{
  community::{HideCommunityFromFeeds, HideCommunityFromFeedsResponse},
  context::LemmyContext,
  utils::local_user_view_from_jwt,
};
use lemmy_db_schema::{
  source::{
    community::{CommunityFollower, CommunityFollowerForm},
    community_hide_from_feeds::{CommunityHideFromFeeds, CommunityHideFromFeedsForm},
  },
  traits::{Followable, HideableFromFeeds},
};
use lemmy_db_views_actor::structs::CommunityView;
use lemmy_utils::error::{LemmyError, LemmyErrorExt, LemmyErrorType};

#[async_trait::async_trait(?Send)]
impl Perform for HideCommunityFromFeeds {
  type Response = HideCommunityFromFeedsResponse;

  #[tracing::instrument(skip(context))]
  async fn perform(
    &self,
    context: &Data<LemmyContext>,
  ) -> Result<HideCommunityFromFeedsResponse, LemmyError> {
    let data: &HideCommunityFromFeeds = self;
    let local_user_view = local_user_view_from_jwt(&data.auth, context).await?;

    let community_id = data.community_id;
    let person_id = local_user_view.person.id;
    let community_hide_from_feeds_form = CommunityHideFromFeedsForm {
      person_id,
      community_id,
    };

    if data.hide_from_feeds {
      CommunityHideFromFeeds::hide_from_feeds(&mut context.pool(), &community_hide_from_feeds_form)
        .await
        .with_lemmy_type(LemmyErrorType::CommunityBlockAlreadyExists)?;

      // Also, unfollow the community, and send a federated unfollow
      let community_follower_form = CommunityFollowerForm {
        community_id: data.community_id,
        person_id,
        pending: false,
      };

      CommunityFollower::unfollow(&mut context.pool(), &community_follower_form)
        .await
        .ok();
    } else {
      CommunityHideFromFeeds::unhide_from_feeds(
        &mut context.pool(),
        &community_hide_from_feeds_form,
      )
      .await
      .with_lemmy_type(LemmyErrorType::CommunityBlockAlreadyExists)?;
    }

    let community_view =
      CommunityView::read(&mut context.pool(), community_id, Some(person_id), None).await?;

    Ok(HideCommunityFromFeedsResponse {
      hidden_from_feeds: data.hide_from_feeds,
      community_view,
    })
  }
}
