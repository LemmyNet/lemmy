use actix_web::web::{Data, Json};
use anyhow::Context;
use lemmy_api_common::{
  community::{GetCommunityResponse, TransferCommunity},
  context::LemmyContext,
  utils::{check_community_user_action, is_admin, is_top_mod},
};
use lemmy_db_schema::{
  source::{
    community::{CommunityModerator, CommunityModeratorForm},
    moderator::{ModTransferCommunity, ModTransferCommunityForm},
  },
  traits::{Crud, Joinable},
};
use lemmy_db_views::structs::LocalUserView;
use lemmy_db_views_actor::structs::{CommunityModeratorView, CommunityView};
use lemmy_utils::{
  error::{LemmyErrorExt, LemmyErrorType, LemmyResult},
  location_info,
};

// TODO: we dont do anything for federation here, it should be updated the next time the community
//       gets fetched. i hope we can get rid of the community creator role soon.
#[tracing::instrument(skip(context))]
pub async fn transfer_community(
  data: Json<TransferCommunity>,
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> LemmyResult<Json<GetCommunityResponse>> {
  let community_id = data.community_id;
  let mut community_mods =
    CommunityModeratorView::for_community(&mut context.pool(), community_id).await?;

  check_community_user_action(&local_user_view.person, community_id, &mut context.pool()).await?;

  // Make sure transferrer is either the top community mod, or an admin
  if !(is_top_mod(&local_user_view, &community_mods).is_ok() || is_admin(&local_user_view).is_ok())
  {
    Err(LemmyErrorType::NotAnAdmin)?
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

  CommunityModerator::delete_for_community(&mut context.pool(), community_id).await?;

  // TODO: this should probably be a bulk operation
  // Re-add the mods, in the new order
  for cmod in &community_mods {
    let community_moderator_form = CommunityModeratorForm {
      community_id: cmod.community.id,
      person_id: cmod.moderator.id,
    };

    CommunityModerator::join(&mut context.pool(), &community_moderator_form)
      .await
      .with_lemmy_type(LemmyErrorType::CommunityModeratorAlreadyExists)?;
  }

  // Mod tables
  let form = ModTransferCommunityForm {
    mod_person_id: local_user_view.person.id,
    other_person_id: data.person_id,
    community_id: data.community_id,
  };

  ModTransferCommunity::create(&mut context.pool(), &form).await?;

  let community_id = data.community_id;
  let person_id = local_user_view.person.id;
  let community_view =
    CommunityView::read(&mut context.pool(), community_id, Some(person_id), false)
      .await
      .with_lemmy_type(LemmyErrorType::CouldntFindCommunity)?;

  let community_id = data.community_id;
  let moderators = CommunityModeratorView::for_community(&mut context.pool(), community_id)
    .await
    .with_lemmy_type(LemmyErrorType::CouldntFindCommunity)?;

  // Return the jwt
  Ok(Json(GetCommunityResponse {
    community_view,
    site: None,
    moderators,
    discussion_languages: vec![],
  }))
}
