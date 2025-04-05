use activitypub_federation::config::Data;
use actix_web::web::Json;
use diesel_async::{scoped_futures::ScopedFutureExt, AsyncConnection};
use lemmy_api_common::{
  community::{AddModToCommunity, AddModToCommunityResponse},
  context::LemmyContext,
  send_activity::{ActivityChannel, SendActivityData},
  utils::check_community_mod_action,
};
use lemmy_db_schema::{
  source::{
    community::{Community, CommunityActions, CommunityModeratorForm},
    local_user::LocalUser,
    mod_log::moderator::{ModAddCommunity, ModAddCommunityForm},
  },
  traits::{Crud, Joinable},
  utils::get_conn,
};
use lemmy_db_views::structs::{CommunityModeratorView, LocalUserView};
use lemmy_utils::error::{LemmyError, LemmyResult};

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
  conn
    .transaction::<_, LemmyError, _>(|conn| {
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
        let form = ModAddCommunityForm {
          mod_person_id: local_user_view.person.id,
          other_person_id: tx_data.person_id,
          community_id: tx_data.community_id,
          removed: Some(!tx_data.added),
        };

        ModAddCommunity::create(&mut conn.into(), &form).await?;

        Ok(())
      }
      .scope_boxed()
    })
    .await?;

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
