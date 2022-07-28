use crate::Perform;
use actix_web::web::Data;
use bcrypt::verify;
use lemmy_api_common::{
  person::{Login, LoginResponse},
  utils::{blocking, check_registration_application, check_user_valid},
};
use lemmy_db_schema::source::site::Site;
use lemmy_db_views::structs::LocalUserView;
use lemmy_utils::{claims::Claims, error::LemmyError, ConnectionId};
use lemmy_websocket::LemmyContext;

#[async_trait::async_trait(?Send)]
impl Perform for Login {
  type Response = LoginResponse;

  #[tracing::instrument(skip(context, _websocket_id))]
  async fn perform(
    &self,
    context: &Data<LemmyContext>,
    _websocket_id: Option<ConnectionId>,
  ) -> Result<LoginResponse, LemmyError> {
    let data: &Login = self;

    // Fetch that username / email
    let username_or_email = data.username_or_email.clone();
    let local_user_view = blocking(context.pool(), move |conn| {
      LocalUserView::find_by_email_or_name(conn, &username_or_email)
    })
    .await?
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

    let site = blocking(context.pool(), Site::read_local_site).await??;
    if site.require_email_verification && !local_user_view.local_user.email_verified {
      return Err(LemmyError::from_message("email_not_verified"));
    }

    check_registration_application(&site, &local_user_view, context.pool()).await?;

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
