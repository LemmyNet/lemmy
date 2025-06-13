use actix_web::{
  web::{Data, Json},
  HttpRequest,
};
use bcrypt::verify;
use lemmy_api_utils::{claims::Claims, context::LemmyContext, utils::password_length_check};
use lemmy_db_schema::source::{local_user::LocalUser, login_token::LoginToken};
use lemmy_db_views_local_user::LocalUserView;
use lemmy_db_views_site::api::{ChangePassword, LoginResponse};
use lemmy_utils::error::{LemmyErrorType, LemmyResult};

pub async fn change_password(
  data: Json<ChangePassword>,
  req: HttpRequest,
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> LemmyResult<Json<LoginResponse>> {
  password_length_check(&data.new_password)?;

  // Make sure passwords match
  if data.new_password != data.new_password_verify {
    Err(LemmyErrorType::PasswordsDoNotMatch)?
  }

  // Check the old password
  let valid: bool = if let Some(password_encrypted) = &local_user_view.local_user.password_encrypted
  {
    verify(&data.old_password, password_encrypted).unwrap_or(false)
  } else {
    data.old_password.is_empty()
  };

  if !valid {
    Err(LemmyErrorType::IncorrectLogin)?
  }

  let local_user_id = local_user_view.local_user.id;
  let new_password = data.new_password.clone();
  let updated_local_user =
    LocalUser::update_password(&mut context.pool(), local_user_id, &new_password).await?;

  LoginToken::invalidate_all(&mut context.pool(), local_user_view.local_user.id).await?;

  // Return the jwt
  Ok(Json(LoginResponse {
    jwt: Some(Claims::generate(updated_local_user.id, req, &context).await?),
    verify_email_sent: false,
    registration_created: false,
  }))
}
