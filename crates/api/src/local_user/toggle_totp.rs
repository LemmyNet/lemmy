use crate::check_totp_2fa_valid;
use actix_web::{
  web::{Data, Json},
  HttpResponse,
};
use lemmy_api_common::{context::LemmyContext, person::ToggleTotp};
use lemmy_db_schema::{
  source::local_user::{LocalUser, LocalUserUpdateForm},
  traits::Crud,
};
use lemmy_db_views::structs::{LocalUserView, SiteView};
use lemmy_utils::error::{LemmyError, LemmyErrorType};

/// Enable or disable two-factor-authentication. The current setting is determined from
/// [LocalUser.totp_2fa_enabled].
///
/// To enable, you need to first call [generate_totp_secret] and then pass a valid token to this
/// function.
///
/// Disabling is only possible if 2FA was previously enabled. Again it is necessary to pass a valid
/// token.
#[tracing::instrument(skip(context))]
pub async fn toggle_totp(
  data: Json<ToggleTotp>,
  local_user_view: LocalUserView,
  context: Data<LemmyContext>,
) -> Result<HttpResponse, LemmyError> {
  let site_view = SiteView::read_local(&mut context.pool()).await?;

  // require valid 2fa token to enable or disable 2fa
  if local_user_view.local_user.totp_2fa_secret.is_none() {
    return Err(LemmyErrorType::MissingTotpToken.into());
  }
  check_totp_2fa_valid(
    &local_user_view.local_user,
    &Some(data.totp_totp_token.clone()),
    &site_view.site.name,
    &local_user_view.person.name,
  )?;

  // toggle the 2fa setting
  let new_totp_state = !local_user_view.local_user.totp_2fa_enabled;
  let mut local_user_form = LocalUserUpdateForm {
    totp_2fa_enabled: Some(new_totp_state),
    ..Default::default()
  };

  // clear totp secret if 2fa is being disabled
  if !new_totp_state {
    local_user_form.totp_2fa_secret = None;
  }

  LocalUser::update(
    &mut context.pool(),
    local_user_view.local_user.id,
    &local_user_form,
  )
  .await?;

  Ok(HttpResponse::Ok().finish())
}
