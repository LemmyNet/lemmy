use crate::PerformCrud;
use actix_web::web::Data;
use bcrypt::verify;
use lemmy_api_common::{
  context::LemmyContext,
  person::{DeleteAccount, DeleteAccountResponse},
  utils::local_user_view_from_jwt,
};
use lemmy_utils::error::{LemmyError, LemmyErrorType};

#[async_trait::async_trait(?Send)]
impl PerformCrud for DeleteAccount {
  type Response = DeleteAccountResponse;

  #[tracing::instrument(skip(self, context))]
  async fn perform(&self, context: &Data<LemmyContext>) -> Result<Self::Response, LemmyError> {
    let data = self;
    let local_user_view = local_user_view_from_jwt(data.auth.as_ref(), context).await?;

    // Verify the password
    let valid: bool = verify(
      &data.password,
      &local_user_view.local_user.password_encrypted,
    )
    .unwrap_or(false);
    if !valid {
      return Err(LemmyErrorType::IncorrectLogin)?;
    }

    Ok(DeleteAccountResponse {})
  }
}
