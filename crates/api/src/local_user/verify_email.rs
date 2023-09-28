use actix_web::{
  web::{Data, Json},
  HttpResponse,
};
use lemmy_api_common::{context::LemmyContext, person::VerifyEmail};
use lemmy_db_schema::{
  source::{
    email_verification::EmailVerification,
    local_user::{LocalUser, LocalUserUpdateForm},
  },
  traits::Crud,
};
use lemmy_utils::error::{LemmyErrorExt, LemmyErrorType, LemmyResult};

pub async fn verify_email(
  data: Json<VerifyEmail>,
  context: Data<LemmyContext>,
) -> LemmyResult<HttpResponse> {
  let token = data.token.clone();
  let verification = EmailVerification::read_for_token(&mut context.pool(), &token)
    .await
    .with_lemmy_type(LemmyErrorType::TokenNotFound)?;

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

  Ok(HttpResponse::Ok().finish())
}
