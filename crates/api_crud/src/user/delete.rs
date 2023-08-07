use activitypub_federation::config::Data;
use actix_web::web::Json;
use bcrypt::verify;
use lemmy_api_common::{
  context::LemmyContext,
  person::{DeleteAccount, DeleteAccountResponse},
  send_activity::{ActivityChannel, SendActivityData},
  utils::local_user_view_from_jwt,
};
use lemmy_utils::error::{LemmyError, LemmyErrorType};

#[tracing::instrument(skip(context))]
pub async fn delete_account(
  data: Json<DeleteAccount>,
  context: Data<LemmyContext>,
) -> Result<Json<DeleteAccountResponse>, LemmyError> {
  let local_user_view = local_user_view_from_jwt(data.auth.as_ref(), &context).await?;

  // Verify the password
  let valid: bool = verify(
    &data.password,
    &local_user_view.local_user.password_encrypted,
  )
  .unwrap_or(false);
  if !valid {
    return Err(LemmyErrorType::IncorrectLogin)?;
  }

  ActivityChannel::submit_activity(
    SendActivityData::DeleteUser(local_user_view.person),
    &context,
  )
  .await?;

  Ok(Json(DeleteAccountResponse {}))
}
