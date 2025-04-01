use actix_web::web::{Data, Json};
use lemmy_api_common::{
  context::LemmyContext,
  person::VerifyEmail,
  utils::{
    get_interface_language_from_settings,
    send_email_to_user,
    send_new_applicant_email_to_admins,
  },
  SuccessResponse,
};
use lemmy_db_schema::source::{
  email_verification::EmailVerification,
  local_user::{LocalUser, LocalUserUpdateForm},
};
use lemmy_db_views::structs::{LocalUserView, SiteView};
use lemmy_utils::error::{LemmyErrorType, LemmyResult};

pub async fn verify_email(
  data: Json<VerifyEmail>,
  context: Data<LemmyContext>,
) -> LemmyResult<Json<SuccessResponse>> {
  let site_view = SiteView::read_local(&mut context.pool())
    .await?
    .ok_or(LemmyErrorType::LocalSiteNotSetup)?;
  let token = data.token.clone();
  let verification = EmailVerification::read_for_token(&mut context.pool(), &token)
    .await?
    .ok_or(LemmyErrorType::TokenNotFound)?;

  let form = LocalUserUpdateForm {
    // necessary in case this is a new signup
    email_verified: Some(true),
    // necessary in case email of an existing user was changed
    email: Some(Some(verification.email)),
    ..Default::default()
  };
  let local_user_id = verification.local_user_id;

  LocalUser::update(&mut context.pool(), local_user_id, &form).await?;

  EmailVerification::delete_old_tokens_for_local_user(&mut context.pool(), local_user_id).await?;

  let local_user_view = LocalUserView::read(&mut context.pool(), local_user_id)
    .await?
    .ok_or(LemmyErrorType::CouldntFindPerson)?;

  // send out notification about registration application to admins if enabled
  if site_view.local_site.application_email_admins {
    send_new_applicant_email_to_admins(
      &local_user_view.person.name,
      &mut context.pool(),
      context.settings(),
    )
    .await?;
  }

  let lang = get_interface_language_from_settings(&local_user_view);
  let subject = lang.email_verified_subject(&local_user_view.person.name);
  let body = lang.email_verified_body();
  send_email_to_user(&local_user_view, &subject, body, context.settings()).await;

  Ok(Json(SuccessResponse::default()))
}
