use activitypub_federation::config::Data;
use actix_web::web::Json;
use diesel_async::scoped_futures::ScopedFutureExt;
use lemmy_api_utils::{
  context::LemmyContext,
  send_activity::{ActivityChannel, SendActivityData},
  utils::{
    check_community_mod_action,
    check_expire_time,
    remove_or_restore_user_data_in_community,
  },
};
use lemmy_db_schema::{
  source::{
    community::{Community, CommunityActions, CommunityPersonBanForm},
    local_user::LocalUser,
    mod_log::moderator::{ModBanFromCommunity, ModBanFromCommunityForm},
  },
  traits::{Bannable, Crud, Followable},
  utils::get_conn,
};
use lemmy_db_views_community::api::{BanFromCommunity, BanFromCommunityResponse};
use lemmy_db_views_local_user::LocalUserView;
use lemmy_db_views_person::PersonView;
use lemmy_utils::{error::LemmyResult, utils::validation::is_valid_body_field};

pub async fn ban_from_community(
  data: Json<BanFromCommunity>,
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> LemmyResult<Json<BanFromCommunityResponse>> {
  let banned_person_id = data.person_id;
  let my_person_id = local_user_view.person.id;
  let expires_at = check_expire_time(data.expires_at)?;
  let local_instance_id = local_user_view.person.instance_id;
  let community = Community::read(&mut context.pool(), data.community_id).await?;

  // Verify that only mods or admins can ban
  check_community_mod_action(&local_user_view, &community, false, &mut context.pool()).await?;

  LocalUser::is_higher_mod_or_admin_check(
    &mut context.pool(),
    data.community_id,
    my_person_id,
    vec![data.person_id],
  )
  .await?;

  if let Some(reason) = &data.reason {
    is_valid_body_field(reason, false)?;
  }

  let community_user_ban_form = CommunityPersonBanForm {
    ban_expires_at: Some(expires_at),
    ..CommunityPersonBanForm::new(data.community_id, data.person_id)
  };

  let pool = &mut context.pool();
  let conn = &mut get_conn(pool).await?;
  let tx_data = data.clone();
  conn
    .run_transaction(|conn| {
      async move {
        if tx_data.ban {
          CommunityActions::ban(&mut conn.into(), &community_user_ban_form).await?;

          // Also unsubscribe them from the community, if they are subscribed
          CommunityActions::unfollow(&mut conn.into(), banned_person_id, tx_data.community_id)
            .await
            .ok();
        } else {
          CommunityActions::unban(&mut conn.into(), &community_user_ban_form).await?;
        }

        // Remove/Restore their data if that's desired
        if tx_data.remove_or_restore_data.unwrap_or(false) {
          let remove_data = tx_data.ban;
          remove_or_restore_user_data_in_community(
            tx_data.community_id,
            my_person_id,
            banned_person_id,
            remove_data,
            &tx_data.reason,
            &mut conn.into(),
          )
          .await?;
        };

        // Mod tables
        let form = ModBanFromCommunityForm {
          mod_person_id: my_person_id,
          other_person_id: tx_data.person_id,
          community_id: tx_data.community_id,
          reason: tx_data.reason.clone(),
          banned: Some(tx_data.ban),
          expires_at,
        };

        ModBanFromCommunity::create(&mut conn.into(), &form).await?;

        Ok(())
      }
      .scope_boxed()
    })
    .await?;

  let person_view = PersonView::read(
    &mut context.pool(),
    data.person_id,
    Some(my_person_id),
    local_instance_id,
    false,
  )
  .await?;

  ActivityChannel::submit_activity(
    SendActivityData::BanFromCommunity {
      moderator: local_user_view.person,
      community_id: data.community_id,
      target: person_view.person.clone(),
      data: data.0.clone(),
    },
    &context,
  )?;

  Ok(Json(BanFromCommunityResponse {
    person_view,
    banned: data.ban,
  }))
}
