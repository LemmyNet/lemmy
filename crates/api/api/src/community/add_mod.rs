use activitypub_federation::config::Data;
use actix_web::web::Json;
use diesel_async::scoped_futures::ScopedFutureExt;
use lemmy_api_utils::{
  context::LemmyContext,
  notify::notify_mod_action,
  send_activity::{ActivityChannel, SendActivityData},
  utils::check_community_mod_action,
};
use lemmy_db_schema::source::{
  community::{Community, CommunityActions, CommunityModeratorForm},
  local_user::LocalUser,
  modlog::{Modlog, ModlogInsertForm},
};
use lemmy_db_views_community::api::{AddModToCommunity, AddModToCommunityResponse};
use lemmy_db_views_community_moderator::CommunityModeratorView;
use lemmy_db_views_local_user::LocalUserView;
use lemmy_diesel_utils::{connection::get_conn, traits::Crud};
use lemmy_utils::error::{LemmyErrorType, LemmyResult};

pub async fn add_mod_to_community(
  data: Json<AddModToCommunity>,
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> LemmyResult<Json<AddModToCommunityResponse>> {
  let community = Community::read(&mut context.pool(), data.community_id).await?;
  // Verify that only mods or admins can add mod
  check_community_mod_action(&local_user_view, &community, false, &mut context.pool()).await?;

  // If it's a mod removal, also check that you're a higher mod.
  if !data.added {
    LocalUser::is_higher_mod_or_admin_check(
      &mut context.pool(),
      community.id,
      local_user_view.person.id,
      vec![data.person_id],
    )
    .await?;

    // Dont allow the last community mod to remove himself
    let mods = CommunityModeratorView::for_community(&mut context.pool(), community.id).await?;
    if !local_user_view.local_user.admin && mods.len() == 1 {
      Err(LemmyErrorType::CannotLeaveMod)?
    }
  }

  // If user is admin and community is remote, explicitly check that he is a
  // moderator. This is necessary because otherwise the action would be rejected
  // by the community's home instance.
  if local_user_view.local_user.admin && !community.local {
    CommunityModeratorView::check_is_community_moderator(
      &mut context.pool(),
      community.id,
      local_user_view.person.id,
    )
    .await?;
  }

  let pool = &mut context.pool();
  let conn = &mut get_conn(pool).await?;
  let tx_data = data.clone();
  let action = conn
    .run_transaction(|conn| {
      async move {
        // Update in local database
        let community_moderator_form =
          CommunityModeratorForm::new(tx_data.community_id, tx_data.person_id);
        if tx_data.added {
          CommunityActions::join(&mut conn.into(), &community_moderator_form).await?;
        } else {
          CommunityActions::leave(&mut conn.into(), &community_moderator_form).await?;
        }

        // Mod tables
        let form = ModlogInsertForm::mod_add_to_community(
          local_user_view.person.id,
          tx_data.community_id,
          tx_data.person_id,
          !tx_data.added,
        );
        Modlog::create(&mut conn.into(), &[form]).await
      }
      .scope_boxed()
    })
    .await?;
  notify_mod_action(action.clone(), &context);

  // Note: in case a remote mod is added, this returns the old moderators list, it will only get
  //       updated once we receive an activity from the community (like `Announce/Add/Moderator`)
  let community_id = data.community_id;
  let moderators = CommunityModeratorView::for_community(&mut context.pool(), community_id).await?;

  ActivityChannel::submit_activity(
    SendActivityData::AddModToCommunity {
      moderator: local_user_view.person,
      community_id: data.community_id,
      target: data.person_id,
      added: data.added,
    },
    &context,
  )?;

  Ok(Json(AddModToCommunityResponse { moderators }))
}
