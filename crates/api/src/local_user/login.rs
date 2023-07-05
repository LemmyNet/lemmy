use crate::Perform;
use actix_web::web::Data;
use bcrypt::verify;
use lemmy_api_common::{
  context::LemmyContext,
  person::{Login, LoginResponse},
  utils::{check_registration_application, check_user_valid},
};
use lemmy_db_views::structs::{LocalUserView, SiteView};
use lemmy_utils::{claims::Claims, error::LemmyError, utils::validation::check_totp_2fa_valid};

#[async_trait::async_trait(?Send)]
impl Perform for Login {
  type Response = LoginResponse;

  #[tracing::instrument(skip(context))]
  async fn perform(&self, context: &Data<LemmyContext>) -> Result<LoginResponse, LemmyError> {
    let data: &Login = self;

    let site_view = SiteView::read_local(context.pool()).await?;

    // Fetch that username / email
    let username_or_email = data.username_or_email.clone();
    let local_user_view = LocalUserView::find_by_email_or_name(context.pool(), &username_or_email)
      .await
      .map_err(|e| LemmyError::from_error_message(e, "couldnt_find_that_username_or_email"))?;

    // Verify the password
    let valid: bool = verify(
      &data.password,
      &local_user_view.local_user.password_encrypted,
    )
    .unwrap_or(false);
    if !valid {
      return Err(LemmyError::from_message("password_incorrect"));
    }
    check_user_valid(
      local_user_view.person.banned,
      local_user_view.person.ban_expires,
      local_user_view.person.deleted,
    )?;

    // Check if the user's email is verified if email verification is turned on
    // However, skip checking verification if the user is an admin
    if !local_user_view.person.admin
      && site_view.local_site.require_email_verification
      && !local_user_view.local_user.email_verified
    {
      return Err(LemmyError::from_message("email_not_verified"));
    }

    check_registration_application(&local_user_view, &site_view.local_site, context.pool()).await?;

    // Check the totp
    check_totp_2fa_valid(
      &local_user_view.local_user.totp_2fa_secret,
      &data.totp_2fa_token,
      &site_view.site.name,
      &local_user_view.person.name,
    )?;

    // Return the jwt
    Ok(LoginResponse {
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
    })
  }
}
