use crate::check_totp_2fa_valid;
use actix_web::{
  web::{Data, Json},
  HttpResponse,
};
use bcrypt::verify;
use lemmy_api_common::{
  claims::Claims,
  context::LemmyContext,
  person::{Login, LoginResponse},
  utils::{check_registration_application, check_user_valid, create_login_cookie},
};
use lemmy_db_views::structs::{LocalUserView, SiteView};
use lemmy_utils::error::{LemmyError, LemmyErrorExt, LemmyErrorType};

#[tracing::instrument(skip(context))]
pub async fn login(
  data: Json<Login>,
  context: Data<LemmyContext>,
) -> Result<HttpResponse, LemmyError> {
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
  check_user_valid(
    local_user_view.person.banned,
    local_user_view.person.ban_expires,
    local_user_view.person.deleted,
  )?;

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

  let jwt = Claims::generate(local_user_view.local_user.id, &context).await?;

  let json = LoginResponse {
    jwt: Some(jwt.clone()),
    verify_email_sent: false,
    registration_created: false,
  };

  let mut res = HttpResponse::Ok().json(json);
  res.add_cookie(&create_login_cookie(jwt))?;
  Ok(res)
}
