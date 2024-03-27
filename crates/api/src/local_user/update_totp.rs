use crate::check_totp_2fa_valid;
use actix_web::web::{Data, Json};
use lemmy_api_common::{
  context::LemmyContext,
  person::{UpdateTotp, UpdateTotpResponse},
};
use lemmy_db_schema::source::local_user::{LocalUser, LocalUserUpdateForm};
use lemmy_db_views::structs::LocalUserView;
use lemmy_utils::error::LemmyError;

/// Enable or disable two-factor-authentication. The current setting is determined from
/// [LocalUser.totp_2fa_enabled].
///
/// To enable, you need to first call [generate_totp_secret] and then pass a valid token to this
/// function.
///
/// Disabling is only possible if 2FA was previously enabled. Again it is necessary to pass a valid
/// token.
#[tracing::instrument(skip(context))]
pub async fn update_totp(
  data: Json<UpdateTotp>,
  local_user_view: LocalUserView,
  context: Data<LemmyContext>,
) -> Result<Json<UpdateTotpResponse>, LemmyError> {
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

  Ok(Json(UpdateTotpResponse {
    enabled: data.enabled,
  }))
}
