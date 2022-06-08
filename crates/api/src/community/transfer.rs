use crate::Perform;
use actix_web::web::Data;
use anyhow::Context;
use lemmy_api_common::{
  community::{GetCommunityResponse, TransferCommunity},
  utils::{blocking, get_local_user_view_from_jwt},
};
use lemmy_db_schema::{
  source::{
    community::{CommunityModerator, CommunityModeratorForm},
    moderator::{ModTransferCommunity, ModTransferCommunityForm},
  },
  traits::{Crud, Joinable},
};
use lemmy_db_views_actor::structs::{CommunityModeratorView, CommunityView, PersonViewSafe};
use lemmy_utils::{error::LemmyError, location_info, ConnectionId};
use lemmy_websocket::LemmyContext;

// TODO: we dont do anything for federation here, it should be updated the next time the community
//       gets fetched. i hope we can get rid of the community creator role soon.
#[async_trait::async_trait(?Send)]
impl Perform for TransferCommunity {
  type Response = GetCommunityResponse;

  #[tracing::instrument(skip(context, _websocket_id))]
  async fn perform(
    &self,
    context: &Data<LemmyContext>,
    _websocket_id: Option<ConnectionId>,
  ) -> Result<GetCommunityResponse, LemmyError> {
    let data: &TransferCommunity = self;
    let local_user_view =
      get_local_user_view_from_jwt(&data.auth, context.pool(), context.secret()).await?;

    let admins = blocking(context.pool(), PersonViewSafe::admins).await??;

    // Fetch the community mods
    let community_id = data.community_id;
    let mut community_mods = blocking(context.pool(), move |conn| {
      CommunityModeratorView::for_community(conn, community_id)
    })
    .await??;

    // Make sure transferrer is either the top community mod, or an admin
    if local_user_view.person.id != community_mods[0].moderator.id
      && !admins
        .iter()
        .map(|a| a.person.id)
        .any(|x| x == local_user_view.person.id)
    {
      return Err(LemmyError::from_message("not_an_admin"));
    }

    // You have to re-do the community_moderator table, reordering it.
    // Add the transferee to the top
    let creator_index = community_mods
      .iter()
      .position(|r| r.moderator.id == data.person_id)
      .context(location_info!())?;
    let creator_person = community_mods.remove(creator_index);
    community_mods.insert(0, creator_person);

    // Delete all the mods
    let community_id = data.community_id;
    blocking(context.pool(), move |conn| {
      CommunityModerator::delete_for_community(conn, community_id)
    })
    .await??;

    // TODO: this should probably be a bulk operation
    // Re-add the mods, in the new order
    for cmod in &community_mods {
      let community_moderator_form = CommunityModeratorForm {
        community_id: cmod.community.id,
        person_id: cmod.moderator.id,
      };

      let join = move |conn: &'_ _| CommunityModerator::join(conn, &community_moderator_form);
      blocking(context.pool(), join)
        .await?
        .map_err(|e| LemmyError::from_error_message(e, "community_moderator_already_exists"))?;
    }

    // Mod tables
    let form = ModTransferCommunityForm {
      mod_person_id: local_user_view.person.id,
      other_person_id: data.person_id,
      community_id: data.community_id,
      removed: Some(false),
    };
    blocking(context.pool(), move |conn| {
      ModTransferCommunity::create(conn, &form)
    })
    .await??;

    let community_id = data.community_id;
    let person_id = local_user_view.person.id;
    let community_view = blocking(context.pool(), move |conn| {
      CommunityView::read(conn, community_id, Some(person_id))
    })
    .await?
    .map_err(|e| LemmyError::from_error_message(e, "couldnt_find_community"))?;

    let community_id = data.community_id;
    let moderators = blocking(context.pool(), move |conn| {
      CommunityModeratorView::for_community(conn, community_id)
    })
    .await?
    .map_err(|e| LemmyError::from_error_message(e, "couldnt_find_community"))?;

    // Return the jwt
    Ok(GetCommunityResponse {
      community_view,
      site: None,
      moderators,
      online: 0,
    })
  }
}
