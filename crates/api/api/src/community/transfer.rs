use actix_web::web::{Data, Json};
use anyhow::Context;
use diesel_async::scoped_futures::ScopedFutureExt;
use lemmy_api_utils::{
  context::LemmyContext,
  notify::notify_mod_action,
  utils::{check_community_user_action, is_admin, is_top_mod},
};
use lemmy_db_schema::{
  source::{
    community::{Community, CommunityActions, CommunityModeratorForm},
    modlog::{Modlog, ModlogInsertForm},
  },
  traits::Crud,
  utils::get_conn,
};
use lemmy_db_views_community::{
  api::{GetCommunityResponse, TransferCommunity},
  CommunityView,
};
use lemmy_db_views_community_moderator::CommunityModeratorView;
use lemmy_db_views_local_user::LocalUserView;
use lemmy_utils::{
  error::{LemmyErrorType, LemmyResult},
  location_info,
};

// TODO: we don't do anything for federation here, it should be updated the next time the community
//       gets fetched. i hope we can get rid of the community creator role soon.

pub async fn transfer_community(
  data: Json<TransferCommunity>,
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> LemmyResult<Json<GetCommunityResponse>> {
  let community = Community::read(&mut context.pool(), data.community_id).await?;
  let mut community_mods =
    CommunityModeratorView::for_community(&mut context.pool(), community.id).await?;

  check_community_user_action(&local_user_view, &community, &mut context.pool()).await?;

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

  let pool = &mut context.pool();
  let conn = &mut get_conn(pool).await?;
  let tx_data = data.clone();
  let action = conn
    .run_transaction(|conn| {
      async move {
        CommunityActions::delete_mods_for_community(&mut conn.into(), community_id).await?;

        // TODO: this should probably be a bulk operation
        // Re-add the mods, in the new order
        for cmod in &community_mods {
          let community_moderator_form =
            CommunityModeratorForm::new(cmod.community.id, cmod.moderator.id);

          CommunityActions::join(&mut conn.into(), &community_moderator_form).await?;
        }

        // Mod tables
        let form = ModlogInsertForm::mod_transfer_community(
          local_user_view.person.id,
          tx_data.community_id,
          tx_data.person_id,
        );
        Modlog::create(&mut conn.into(), &[form]).await
      }
      .scope_boxed()
    })
    .await?;
  notify_mod_action(action.clone(), &context);

  let community_id = data.community_id;
  let community_view = CommunityView::read(
    &mut context.pool(),
    community_id,
    Some(&local_user_view.local_user),
    false,
  )
  .await?;

  let community_id = data.community_id;
  let moderators = CommunityModeratorView::for_community(&mut context.pool(), community_id).await?;

  // Return the jwt
  Ok(Json(GetCommunityResponse {
    community_view,
    site: None,
    moderators,
    discussion_languages: vec![],
  }))
}
