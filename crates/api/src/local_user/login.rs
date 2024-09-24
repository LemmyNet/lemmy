use crate::check_totp_2fa_valid;
use actix_web::{
  web::{Data, Json},
  HttpRequest,
};
use bcrypt::verify;
use lemmy_api_common::{
  claims::Claims,
  context::LemmyContext,
  person::{Login, LoginResponse},
  utils::{check_email_verified, check_registration_application, check_user_valid},
};
use lemmy_db_views::structs::{LocalUserView, SiteView};
use lemmy_utils::error::{LemmyErrorType, LemmyResult};

#[tracing::instrument(skip(context))]
pub async fn login(
  data: Json<Login>,
  req: HttpRequest,
  context: Data<LemmyContext>,
) -> LemmyResult<Json<LoginResponse>> {
  let site_view = SiteView::read_local(&mut context.pool()).await?;

  // Fetch that username / email
  let username_or_email = data.username_or_email.clone();
  let local_user_view =
    LocalUserView::find_by_email_or_name(&mut context.pool(), &username_or_email).await?;

  // Verify the password
  let valid: bool = local_user_view
    .local_user
    .password_encrypted
    .as_ref()
    .and_then(|password_encrypted| verify(&data.password, password_encrypted).ok())
    .unwrap_or(false);
  if !valid {
    Err(LemmyErrorType::IncorrectLogin)?
  }
  check_user_valid(&local_user_view.person)?;
  check_email_verified(&local_user_view, &site_view)?;

  check_registration_application(&local_user_view, &site_view.local_site, &mut context.pool())
    .await?;

  // Check the totp if enabled
  if local_user_view.local_user.totp_2fa_enabled {
    check_totp_2fa_valid(
      &local_user_view,
      &data.totp_2fa_token,
      &context.settings().hostname,
    )?;
  }

  let jwt = Claims::generate(local_user_view.local_user.id, req, &context).await?;

  Ok(Json(LoginResponse {
    jwt: Some(jwt.clone()),
    verify_email_sent: false,
    registration_created: false,
  }))
}
