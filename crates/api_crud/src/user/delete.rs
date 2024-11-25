use activitypub_federation::config::Data;
use actix_web::web::Json;
use bcrypt::verify;
use lemmy_api_common::{
  context::LemmyContext,
  person::DeleteAccount,
  send_activity::{ActivityChannel, SendActivityData},
  utils::purge_user_account,
  SuccessResponse,
};
use lemmy_db_schema::source::{
  login_token::LoginToken,
  oauth_account::OAuthAccount,
  person::Person,
};
use lemmy_db_views::structs::LocalUserView;
use lemmy_utils::error::{LemmyErrorType, LemmyResult};

#[tracing::instrument(skip(context))]
pub async fn delete_account(
  data: Json<DeleteAccount>,
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> LemmyResult<Json<SuccessResponse>> {
  // Verify the password
  let valid: bool = local_user_view
    .local_user
    .password_encrypted
    .as_ref()
    .and_then(|password_encrypted| verify(&data.password, password_encrypted).ok())
    .unwrap_or(false);
  if !valid {
    Err(LemmyErrorType::IncorrectLogin)?
  }

  if data.delete_content {
    purge_user_account(local_user_view.person.id, &context).await?;
  } else {
    OAuthAccount::delete_user_accounts(&mut context.pool(), local_user_view.local_user.id).await?;
    Person::delete_account(&mut context.pool(), local_user_view.person.id).await?;
  }

  LoginToken::invalidate_all(&mut context.pool(), local_user_view.local_user.id).await?;

  ActivityChannel::submit_activity(
    SendActivityData::DeleteUser(local_user_view.person, data.delete_content),
    &context,
  )?;

  Ok(Json(SuccessResponse::default()))
}
