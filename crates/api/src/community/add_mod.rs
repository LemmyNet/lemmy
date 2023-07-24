use crate::Perform;
use actix_web::web::Data;
use lemmy_api_common::{
  community::{AddModToCommunity, AddModToCommunityResponse},
  context::LemmyContext,
  utils::{is_mod_or_admin, local_user_view_from_jwt},
};
use lemmy_db_schema::{
  source::{
    community::{Community, CommunityModerator, CommunityModeratorForm},
    moderator::{ModAddCommunity, ModAddCommunityForm},
  },
  traits::{Crud, Joinable},
};
use lemmy_db_views_actor::structs::CommunityModeratorView;
use lemmy_utils::error::{LemmyError, LemmyErrorExt, LemmyErrorType};

#[async_trait::async_trait(?Send)]
impl Perform for AddModToCommunity {
  type Response = AddModToCommunityResponse;

  #[tracing::instrument(skip(context))]
  async fn perform(
    &self,
    context: &Data<LemmyContext>,
  ) -> Result<AddModToCommunityResponse, LemmyError> {
    let data: &AddModToCommunity = self;
    let local_user_view = local_user_view_from_jwt(&data.auth, context).await?;

    let community_id = data.community_id;

    // Verify that only mods or admins can add mod
    is_mod_or_admin(&mut context.pool(), local_user_view.person.id, community_id).await?;
    let community = Community::read(&mut context.pool(), community_id).await?;
    if local_user_view.person.admin && !community.local {
      return Err(LemmyErrorType::NotAModerator)?;
    }

    // Update in local database
    let community_moderator_form = CommunityModeratorForm {
      community_id: data.community_id,
      person_id: data.person_id,
    };
    if data.added {
      CommunityModerator::join(&mut context.pool(), &community_moderator_form)
        .await
        .with_lemmy_type(LemmyErrorType::CommunityModeratorAlreadyExists)?;
    } else {
      CommunityModerator::leave(&mut context.pool(), &community_moderator_form)
        .await
        .with_lemmy_type(LemmyErrorType::CommunityModeratorAlreadyExists)?;
    }

    // Mod tables
    let form = ModAddCommunityForm {
      mod_person_id: local_user_view.person.id,
      other_person_id: data.person_id,
      community_id: data.community_id,
      removed: Some(!data.added),
    };

    ModAddCommunity::create(&mut context.pool(), form).await?;

    // Note: in case a remote mod is added, this returns the old moderators list, it will only get
    //       updated once we receive an activity from the community (like `Announce/Add/Moderator`)
    let community_id = data.community_id;
    let moderators =
      CommunityModeratorView::for_community(&mut context.pool(), community_id).await?;

    Ok(AddModToCommunityResponse { moderators })
  }
}
