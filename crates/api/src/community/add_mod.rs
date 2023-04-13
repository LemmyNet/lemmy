use crate::Perform;
use actix_web::web::Data;
use lemmy_api_common::{
  community::{AddModToCommunity, AddModToCommunityResponse},
  context::LemmyContext,
  utils::{get_local_user_view_from_jwt, is_mod_or_admin},
  websocket::UserOperation,
};
use lemmy_db_schema::{
  source::{
    community::{Community, CommunityModerator, CommunityModeratorForm},
    moderator::{ModAddCommunity, ModAddCommunityForm},
  },
  traits::{Crud, Joinable},
};
use lemmy_db_views_actor::structs::CommunityModeratorView;
use lemmy_utils::{error::LemmyError, ConnectionId};

#[async_trait::async_trait(?Send)]
impl Perform for AddModToCommunity {
  type Response = AddModToCommunityResponse;

  #[tracing::instrument(skip(context, websocket_id))]
  async fn perform(
    &self,
    context: &Data<LemmyContext>,
    websocket_id: Option<ConnectionId>,
  ) -> Result<AddModToCommunityResponse, LemmyError> {
    let data: &AddModToCommunity = self;
    let local_user_view =
      get_local_user_view_from_jwt(&data.auth, context.pool(), context.secret()).await?;

    let community_id = data.community_id;

    // Verify that only mods or admins can add mod
    is_mod_or_admin(context.pool(), local_user_view.person.id, community_id).await?;
    let community = Community::read(context.pool(), community_id).await?;
    if local_user_view.person.admin && !community.local {
      return Err(LemmyError::from_message("not_a_moderator"));
    }

    // Update in local database
    let community_moderator_form = CommunityModeratorForm {
      community_id: data.community_id,
      person_id: data.person_id,
    };
    if data.added {
      CommunityModerator::join(context.pool(), &community_moderator_form)
        .await
        .map_err(|e| LemmyError::from_error_message(e, "community_moderator_already_exists"))?;
    } else {
      CommunityModerator::leave(context.pool(), &community_moderator_form)
        .await
        .map_err(|e| LemmyError::from_error_message(e, "community_moderator_already_exists"))?;
    }

    // Mod tables
    let form = ModAddCommunityForm {
      mod_person_id: local_user_view.person.id,
      other_person_id: data.person_id,
      community_id: data.community_id,
      removed: Some(!data.added),
    };

    ModAddCommunity::create(context.pool(), &form).await?;

    // Note: in case a remote mod is added, this returns the old moderators list, it will only get
    //       updated once we receive an activity from the community (like `Announce/Add/Moderator`)
    let community_id = data.community_id;
    let moderators = CommunityModeratorView::for_community(context.pool(), community_id).await?;

    let res = AddModToCommunityResponse { moderators };
    context.send_mod_ws_message(
      &UserOperation::AddModToCommunity,
      &res,
      community_id,
      websocket_id,
    )?;

    Ok(res)
  }
}
