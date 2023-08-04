use activitypub_federation::config::Data;
use actix_web::web::Json;
use bcrypt::verify;
use lemmy_api_common::{
  context::LemmyContext,
  person::{DeleteAccount, DeleteAccountResponse},
  send_activity::{ActivityChannel, SendActivityData},
  utils::{local_user_view_from_jwt, purge_user_account},
};
use lemmy_db_schema::source::person::Person;
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

  if data.delete_content {
    purge_user_account(
      local_user_view.person.id,
      &mut context.pool(),
      &context.settings(),
      context.client(),
    )
    .await?;
  } else {
    Person::delete_account(&mut context.pool(), local_user_view.person.id).await?;
  }

  ActivityChannel::submit_activity(
    SendActivityData::DeleteUser(local_user_view.person, data.delete_content),
    &context,
  )
  .await?;

  Ok(Json(DeleteAccountResponse {}))
}
