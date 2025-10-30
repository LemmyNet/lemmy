use activitypub_federation::config::Data;
use actix_web::web::Json;
use lemmy_api_utils::{
  context::LemmyContext,
  send_activity::{ActivityChannel, SendActivityData},
  utils::{is_admin, purge_user_account},
};
use lemmy_db_schema::{
  source::{
    instance::{InstanceActions, InstanceBanForm},
    local_user::LocalUser,
    modlog::{Modlog, ModlogInsertForm},
    person::Person,
  },
  traits::{Bannable, Crud},
};
use lemmy_db_views_local_user::LocalUserView;
use lemmy_db_views_person::api::PurgePerson;
use lemmy_db_views_site::api::SuccessResponse;
use lemmy_utils::error::LemmyResult;

pub async fn purge_person(
  data: Json<PurgePerson>,
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> LemmyResult<Json<SuccessResponse>> {
  let local_instance_id = local_user_view.person.instance_id;

  // Only let admin purge an item
  is_admin(&local_user_view)?;

  // Also check that you're a higher admin
  LocalUser::is_higher_admin_check(
    &mut context.pool(),
    local_user_view.person.id,
    vec![data.person_id],
  )
  .await?;

  let person = Person::read(&mut context.pool(), data.person_id).await?;

  ActivityChannel::submit_activity(
    SendActivityData::BanFromSite {
      moderator: local_user_view.person.clone(),
      banned_user: person,
      reason: data.reason.clone(),
      remove_or_restore_data: Some(true),
      ban: true,
      expires_at: None,
    },
    &context,
  )?;

  // Clear profile data.
  purge_user_account(data.person_id, local_instance_id, &context).await?;

  // Keep person record, but mark as banned to prevent login or refetching from home instance.
  InstanceActions::ban(
    &mut context.pool(),
    &InstanceBanForm::new(data.person_id, local_instance_id, None),
  )
  .await?;

  // Mod tables
  let form = ModlogInsertForm::admin_purge_person(local_user_view.person.id, &data.reason);
  Modlog::create(&mut context.pool(), &[form]).await?;

  Ok(Json(SuccessResponse::default()))
}
