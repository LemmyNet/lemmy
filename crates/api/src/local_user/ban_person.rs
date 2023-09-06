use activitypub_federation::config::Data;
use actix_web::web::Json;
use lemmy_api_common::{
  context::LemmyContext,
  person::{BanPerson, BanPersonResponse},
  send_activity::{ActivityChannel, SendActivityData},
  utils::{is_admin, remove_user_data, sanitize_html_opt},
};
use lemmy_db_schema::{
  source::{
    moderator::{ModBan, ModBanForm},
    person::{Person, PersonUpdateForm},
  },
  traits::Crud,
};
use lemmy_db_views::structs::LocalUserView;
use lemmy_db_views_actor::structs::PersonView;
use lemmy_utils::{
  error::{LemmyError, LemmyErrorExt, LemmyErrorType},
  utils::{time::naive_from_unix, validation::is_valid_body_field},
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

  let expires = data.expires.map(naive_from_unix);

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

  // Remove their data if that's desired
  let remove_data = data.remove_data.unwrap_or(false);
  if remove_data {
    remove_user_data(person.id, &context).await?;
  }

  // Mod tables
  let form = ModBanForm {
    mod_person_id: local_user_view.person.id,
    other_person_id: data.person_id,
    reason: sanitize_html_opt(&data.reason),
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
