use crate::Perform;
use actix_web::web::Data;
use lemmy_api_common::{
  context::LemmyContext,
  person::{VerifyEmail, VerifyEmailResponse},
};
use lemmy_db_schema::{
  source::{
    email_verification::EmailVerification,
    local_user::{LocalUser, LocalUserUpdateForm},
  },
  traits::Crud,
};
use lemmy_utils::error::{LemmyError, LemmyErrorExt, LemmyErrorType};

#[async_trait::async_trait(?Send)]
impl Perform for VerifyEmail {
  type Response = VerifyEmailResponse;

  async fn perform(&self, context: &Data<LemmyContext>) -> Result<Self::Response, LemmyError> {
    let token = self.token.clone();
    let verification = EmailVerification::read_for_token(&mut context.pool(), &token)
      .await
      .with_lemmy_type(LemmyErrorType::TokenNotFound)?;

    let form = LocalUserUpdateForm::builder()
      // necessary in case this is a new signup
      .email_verified(Some(true))
      // necessary in case email of an existing user was changed
      .email(Some(Some(verification.email)))
      .build();
    let local_user_id = verification.local_user_id;

    LocalUser::update(&mut context.pool(), local_user_id, &form).await?;

    EmailVerification::delete_old_tokens_for_local_user(&mut context.pool(), local_user_id).await?;

    Ok(VerifyEmailResponse {})
  }
}
