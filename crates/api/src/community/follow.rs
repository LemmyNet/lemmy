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
use lemmy_utils::error::{LemmyError, LemmyErrorExt, LemmyErrorType};

#[async_trait::async_trait(?Send)]
impl Perform for FollowCommunity {
  type Response = CommunityResponse;

  #[tracing::instrument(skip(context))]
  async fn perform(&self, context: &Data<LemmyContext>) -> Result<CommunityResponse, LemmyError> {
    let data: &FollowCommunity = self;
    let local_user_view = local_user_view_from_jwt(&data.auth, context).await?;

    let community_id = data.community_id;
    let community = Community::read(&mut context.pool(), community_id).await?;
    let mut community_follower_form = CommunityFollowerForm {
      community_id: data.community_id,
      person_id: local_user_view.person.id,
      pending: false,
    };

    if data.follow {
      if community.local {
        check_community_ban(local_user_view.person.id, community_id, &mut context.pool()).await?;
        check_community_deleted_or_removed(community_id, &mut context.pool()).await?;

        CommunityFollower::follow(&mut context.pool(), &community_follower_form)
          .await
          .with_lemmy_type(LemmyErrorType::CommunityFollowerAlreadyExists)?;
      } else {
        // Mark as pending, the actual federation activity is sent via `SendActivity` handler
        community_follower_form.pending = true;
        CommunityFollower::follow(&mut context.pool(), &community_follower_form)
          .await
          .with_lemmy_type(LemmyErrorType::CommunityFollowerAlreadyExists)?;
      }
    }
    if !data.follow {
      CommunityFollower::unfollow(&mut context.pool(), &community_follower_form)
        .await
        .with_lemmy_type(LemmyErrorType::CommunityFollowerAlreadyExists)?;
    }

    let community_id = data.community_id;
    let person_id = local_user_view.person.id;
    let community_view =
      CommunityView::read(&mut context.pool(), community_id, Some(person_id), None).await?;
    let discussion_languages = CommunityLanguage::read(&mut context.pool(), community_id).await?;

    Ok(Self::Response {
      community_view,
      discussion_languages,
    })
  }
}
