use activitypub_federation::config::Data;
use actix_web::web::Json;
use lemmy_api_utils::{
  context::LemmyContext,
  send_activity::{ActivityChannel, SendActivityData},
  utils::{check_expire_time, is_admin, remove_or_restore_user_data},
};
use lemmy_db_schema::{
  source::{
    instance::{InstanceActions, InstanceBanForm},
    local_user::LocalUser,
    mod_log::moderator::{ModBan, ModBanForm},
  },
  traits::{Bannable, Crud},
};
use lemmy_db_views_local_user::LocalUserView;
use lemmy_db_views_person::{
  api::{BanPerson, BanPersonResponse},
  PersonView,
};
use lemmy_utils::{error::LemmyResult, utils::validation::is_valid_body_field};

pub async fn ban_from_site(
  data: Json<BanPerson>,
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> LemmyResult<Json<BanPersonResponse>> {
  let local_instance_id = local_user_view.person.instance_id;
  let my_person_id = local_user_view.person.id;

  // Make sure user is an admin
  is_admin(&local_user_view)?;

  // Also make sure you're a higher admin than the target
  LocalUser::is_higher_admin_check(&mut context.pool(), my_person_id, vec![data.person_id]).await?;

  if let Some(reason) = &data.reason {
    is_valid_body_field(reason, false)?;
  }

  let expires_at = check_expire_time(data.expires_at)?;

  let form = InstanceBanForm::new(
    data.person_id,
    local_user_view.person.instance_id,
    expires_at,
  );
  if data.ban {
    InstanceActions::ban(&mut context.pool(), &form).await?;
  } else {
    InstanceActions::unban(&mut context.pool(), &form).await?;
  }

  // Remove their data if that's desired
  if data.remove_or_restore_data.unwrap_or(false) {
    let removed = data.ban;
    remove_or_restore_user_data(
      my_person_id,
      data.person_id,
      removed,
      &data.reason,
      &context,
    )
    .await?;
  };

  // Mod tables
  let form = ModBanForm {
    mod_person_id: my_person_id,
    other_person_id: data.person_id,
    reason: data.reason.clone(),
    banned: Some(data.ban),
    expires_at,
    instance_id: local_user_view.person.instance_id,
  };

  ModBan::create(&mut context.pool(), &form).await?;

  let person_view = PersonView::read(
    &mut context.pool(),
    data.person_id,
    Some(my_person_id),
    local_instance_id,
    false,
  )
  .await?;

  ActivityChannel::submit_activity(
    SendActivityData::BanFromSite {
      moderator: local_user_view.person,
      banned_user: person_view.person.clone(),
      reason: data.reason.clone(),
      remove_or_restore_data: data.remove_or_restore_data,
      ban: data.ban,
      expires_at: data.expires_at,
    },
    &context,
  )?;

  Ok(Json(BanPersonResponse {
    person_view,
    banned: data.ban,
  }))
}
