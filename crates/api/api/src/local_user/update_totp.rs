use crate::check_totp_2fa_valid;
use actix_web::web::{Data, Json};
use lemmy_api_utils::{context::LemmyContext, utils::check_local_user_valid};
use lemmy_db_schema::source::local_user::{LocalUser, LocalUserUpdateForm};
use lemmy_db_views_local_user::LocalUserView;
use lemmy_db_views_site::api::{EditTotp, EditTotpResponse};
use lemmy_utils::error::LemmyResult;

/// Enable or disable two-factor-authentication. The current setting is determined from
/// [LocalUser.totp_2fa_enabled].
///
/// To enable, you need to first call [generate_totp_secret] and then pass a valid token to this
/// function.
///
/// Disabling is only possible if 2FA was previously enabled. Again it is necessary to pass a valid
/// token.
pub async fn edit_totp(
  Json(data): Json<EditTotp>,
  local_user_view: LocalUserView,
  context: Data<LemmyContext>,
) -> LemmyResult<Json<EditTotpResponse>> {
  check_local_user_valid(&local_user_view)?;
  check_totp_2fa_valid(
    &local_user_view,
    &Some(data.totp_token.clone()),
    &context.settings().hostname,
  )?;

  // toggle the 2fa setting
  let local_user_form = LocalUserUpdateForm {
    totp_2fa_enabled: Some(data.enabled),
    // if totp is enabled, leave unchanged. otherwise clear secret
    totp_2fa_secret: if data.enabled { None } else { Some(None) },
    ..Default::default()
  };

  LocalUser::update(
    &mut context.pool(),
    local_user_view.local_user.id,
    &local_user_form,
  )
  .await?;

  Ok(Json(EditTotpResponse {
    enabled: data.enabled,
  }))
}
