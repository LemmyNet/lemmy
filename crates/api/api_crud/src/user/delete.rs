use activitypub_federation::config::Data;
use actix_web::web::Json;
use bcrypt::verify;
use lemmy_api_utils::{
  context::LemmyContext,
  plugins::{plugin_hook_after, plugin_hook_before},
  send_activity::{ActivityChannel, SendActivityData},
  utils::purge_user_account,
};
use lemmy_db_schema::source::{
  community::CommunityActions,
  login_token::LoginToken,
  oauth_account::OAuthAccount,
  person::Person,
};
use lemmy_db_views_local_user::LocalUserView;
use lemmy_db_views_site::api::{DeleteAccount, DeleteUserForm, SuccessResponse};
use lemmy_utils::error::{LemmyErrorType, LemmyResult};

pub async fn delete_account(
  Json(data): Json<DeleteAccount>,
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> LemmyResult<Json<SuccessResponse>> {
  let local_instance_id = local_user_view.person.instance_id;

  // Verify the password
  let valid: bool = local_user_view
    .local_user
    .password_encrypted
    .as_ref()
    .and_then(|password_encrypted| verify(&data.password, password_encrypted).ok())
    .unwrap_or(false);
  if !valid {
    return Err(LemmyErrorType::IncorrectLogin.into());
  }

  let mut form = DeleteUserForm {
    person_id: local_user_view.person.id,
    delete_content: data.delete_content,
  };

  form = plugin_hook_before("local_user_before_delete", form).await?;
  if form.delete_content {
    purge_user_account(local_user_view.person.id, local_instance_id, &context).await?;
  } else {
    // These are already run in purge_user_account,
    // but should be done anyway even if delete_content is false
    OAuthAccount::delete_user_accounts(&mut context.pool(), local_user_view.local_user.id).await?;
    CommunityActions::leave_mod_team_for_all_communities(
      &mut context.pool(),
      local_user_view.person.id,
    )
    .await?;
    Person::delete_account(
      &mut context.pool(),
      local_user_view.person.id,
      local_instance_id,
    )
    .await?;
  }
  plugin_hook_after("local_user_after_delete", &form);

  LoginToken::invalidate_all(&mut context.pool(), local_user_view.local_user.id).await?;

  ActivityChannel::submit_activity(
    SendActivityData::DeleteUser(local_user_view.person, data.delete_content),
    &context,
  )?;

  Ok(Json(SuccessResponse::default()))
}
