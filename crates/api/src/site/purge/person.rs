use crate::ban_nonlocal_user_from_local_communities;
use activitypub_federation::config::Data;
use actix_web::web::Json;
use lemmy_api_common::{
  context::LemmyContext,
  send_activity::{ActivityChannel, SendActivityData},
  site::PurgePerson,
  utils::{is_admin, purge_user_account},
  SuccessResponse,
};
use lemmy_db_schema::{
  source::{
    local_user::LocalUser,
    moderator::{AdminPurgePerson, AdminPurgePersonForm},
    person::{Person, PersonUpdateForm},
  },
  traits::Crud,
};
use lemmy_db_views::structs::LocalUserView;
use lemmy_utils::{error::LemmyResult, LemmyErrorType};

#[tracing::instrument(skip(context))]
pub async fn purge_person(
  data: Json<PurgePerson>,
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> LemmyResult<Json<SuccessResponse>> {
  // Only let admin purge an item
  is_admin(&local_user_view)?;

  // Also check that you're a higher admin
  LocalUser::is_higher_admin_check(
    &mut context.pool(),
    local_user_view.person.id,
    vec![data.person_id],
  )
  .await?;

  let person = Person::read(&mut context.pool(), data.person_id)
    .await?
    .ok_or(LemmyErrorType::CouldntFindPerson)?;

  ban_nonlocal_user_from_local_communities(
    &local_user_view,
    &person,
    true,
    &data.reason,
    &Some(true),
    &None,
    &context,
  )
  .await?;

  // Clear profile data.
  purge_user_account(data.person_id, &context).await?;

  // Keep person record, but mark as banned to prevent login or refetching from home instance.
  let person = Person::update(
    &mut context.pool(),
    data.person_id,
    &PersonUpdateForm {
      banned: Some(true),
      ..Default::default()
    },
  )
  .await?;

  // Mod tables
  let form = AdminPurgePersonForm {
    admin_person_id: local_user_view.person.id,
    reason: data.reason.clone(),
  };
  AdminPurgePerson::create(&mut context.pool(), &form).await?;

  ActivityChannel::submit_activity(
    SendActivityData::BanFromSite {
      moderator: local_user_view.person,
      banned_user: person,
      reason: data.reason.clone(),
      remove_data: Some(true),
      ban: true,
      expires: None,
    },
    &context,
  )
  .await?;

  Ok(Json(SuccessResponse::default()))
}
