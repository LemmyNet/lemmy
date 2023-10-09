use crate::check_totp_2fa_valid;
use actix_web::web::{Data, Json};
use bcrypt::verify;
use lemmy_api_common::{
  context::LemmyContext,
  person::{Login, LoginResponse},
  utils::check_user_valid,
};
use lemmy_db_schema::{
  source::{local_site::LocalSite, registration_application::RegistrationApplication},
  utils::DbPool,
  RegistrationMode,
};
use lemmy_db_views::structs::{LocalUserView, SiteView};
use lemmy_utils::{
  claims::Claims,
  error::{LemmyError, LemmyErrorExt, LemmyErrorType},
};

#[tracing::instrument(skip(context))]
pub async fn login(
  data: Json<Login>,
  context: Data<LemmyContext>,
) -> Result<Json<LoginResponse>, LemmyError> {
  let site_view = SiteView::read_local(&mut context.pool()).await?;

  // Fetch that username / email
  let username_or_email = data.username_or_email.clone();
  let local_user_view =
    LocalUserView::find_by_email_or_name(&mut context.pool(), &username_or_email)
      .await
      .with_lemmy_type(LemmyErrorType::IncorrectLogin)?;

  // Verify the password
  let valid: bool = verify(
    &data.password,
    &local_user_view.local_user.password_encrypted,
  )
  .unwrap_or(false);
  if !valid {
    Err(LemmyErrorType::IncorrectLogin)?
  }
  check_user_valid(&local_user_view.person)?;

  // Check if the user's email is verified if email verification is turned on
  // However, skip checking verification if the user is an admin
  if !local_user_view.local_user.admin
    && site_view.local_site.require_email_verification
    && !local_user_view.local_user.email_verified
  {
    Err(LemmyErrorType::EmailNotVerified)?
  }

  check_registration_application(&local_user_view, &site_view.local_site, &mut context.pool())
    .await?;

  // Check the totp if enabled
  if local_user_view.local_user.totp_2fa_enabled {
    check_totp_2fa_valid(&local_user_view, &data.totp_2fa_token, &site_view.site.name)?;
  }

  // Return the jwt
  Ok(Json(LoginResponse {
    jwt: Some(
      Claims::jwt(
        local_user_view.local_user.id.0,
        &context.secret().jwt_secret,
        &context.settings().hostname,
      )?
      .into(),
    ),
    verify_email_sent: false,
    registration_created: false,
  }))
}

async fn check_registration_application(
  local_user_view: &LocalUserView,
  local_site: &LocalSite,
  pool: &mut DbPool<'_>,
) -> Result<(), LemmyError> {
  if (local_site.registration_mode == RegistrationMode::RequireApplication
    || local_site.registration_mode == RegistrationMode::Closed)
    && !local_user_view.local_user.accepted_application
    && !local_user_view.local_user.admin
  {
    // Fetch the registration application. If no admin id is present its still pending. Otherwise it
    // was processed (either accepted or denied).
    let local_user_id = local_user_view.local_user.id;
    let registration = RegistrationApplication::find_by_local_user_id(pool, local_user_id).await?;
    if registration.admin_id.is_some() {
      Err(LemmyErrorType::RegistrationDenied(registration.deny_reason))?
    } else {
      Err(LemmyErrorType::RegistrationApplicationIsPending)?
    }
  }
  Ok(())
}
