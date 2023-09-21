use activitypub_federation::config::Data;
use actix_web::web::Json;
use bcrypt::verify;
use lemmy_api_common::{
  context::LemmyContext,
  person::{DeleteAccount, DeleteAccountResponse},
  send_activity::{ActivityChannel, SendActivityData},
  utils::purge_user_account,
};
use lemmy_db_schema::source::{login_token::LoginToken, person::Person};
use lemmy_db_views::structs::LocalUserView;
use lemmy_utils::error::{LemmyError, LemmyErrorType};

#[tracing::instrument(skip(context))]
pub async fn delete_account(
  data: Json<DeleteAccount>,
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> Result<Json<DeleteAccountResponse>, LemmyError> {
  // Verify the password
  let valid: bool = verify(
    &data.password,
    &local_user_view.local_user.password_encrypted,
  )
  .unwrap_or(false);
  if !valid {
    Err(LemmyErrorType::IncorrectLogin)?
  }

  if data.delete_content {
    purge_user_account(local_user_view.person.id, &context).await?;
  } else {
    Person::delete_account(&mut context.pool(), local_user_view.person.id).await?;
  }

  LoginToken::invalidate_all(&mut context.pool(), local_user_view.local_user.id).await?;

  ActivityChannel::submit_activity(
    SendActivityData::DeleteUser(local_user_view.person, data.delete_content),
    &context,
  )
  .await?;

  Ok(Json(DeleteAccountResponse {}))
}
