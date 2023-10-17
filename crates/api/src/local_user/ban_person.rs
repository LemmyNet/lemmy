use activitypub_federation::config::Data;
use actix_web::web::Json;
use lemmy_api_common::{
  context::LemmyContext,
  person::{BanPerson, BanPersonResponse},
  send_activity::{ActivityChannel, SendActivityData},
  utils::{check_expire_time, is_admin, remove_user_data},
};
use lemmy_db_schema::{
  source::{
    login_token::LoginToken,
    moderator::{ModBan, ModBanForm},
    person::{Person, PersonUpdateForm},
  },
  traits::Crud,
};
use lemmy_db_views::structs::LocalUserView;
use lemmy_db_views_actor::structs::PersonView;
use lemmy_utils::{
  error::{LemmyError, LemmyErrorExt, LemmyErrorType},
  utils::validation::is_valid_body_field,
};

#[tracing::instrument(skip(context))]
pub async fn ban_from_site(
  data: Json<BanPerson>,
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> Result<Json<BanPersonResponse>, LemmyError> {
  // Make sure user is an admin
  is_admin(&local_user_view)?;

  is_valid_body_field(&data.reason, false)?;

  let expires = check_expire_time(data.expires)?;

  let person = Person::update(
    &mut context.pool(),
    data.person_id,
    &PersonUpdateForm {
      banned: Some(data.ban),
      ban_expires: Some(expires),
      ..Default::default()
    },
  )
  .await
  .with_lemmy_type(LemmyErrorType::CouldntUpdateUser)?;

  let local_user_id = LocalUserView::read_person(&mut context.pool(), data.person_id)
    .await?
    .local_user
    .id;
  LoginToken::invalidate_all(&mut context.pool(), local_user_id).await?;

  // Remove their data if that's desired
  let remove_data = data.remove_data.unwrap_or(false);
  if remove_data {
    remove_user_data(person.id, &context).await?;
  }

  // Mod tables
  let form = ModBanForm {
    mod_person_id: local_user_view.person.id,
    other_person_id: data.person_id,
    reason: data.reason.clone(),
    banned: Some(data.ban),
    expires,
  };

  ModBan::create(&mut context.pool(), &form).await?;

  let person_view = PersonView::read(&mut context.pool(), data.person_id).await?;

  ActivityChannel::submit_activity(
    SendActivityData::BanFromSite(
      local_user_view.person,
      person_view.person.clone(),
      data.0.clone(),
    ),
    &context,
  )
  .await?;

  Ok(Json(BanPersonResponse {
    person_view,
    banned: data.ban,
  }))
}
