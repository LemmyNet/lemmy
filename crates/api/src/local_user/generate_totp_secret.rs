use crate::{build_totp_2fa, generate_totp_2fa_secret};
use activitypub_federation::config::Data;
use actix_web::web::Json;
use lemmy_api_common::{
  context::LemmyContext,
  person::GenerateTotpSecretResponse,
  sensitive::Sensitive,
};
use lemmy_db_schema::source::local_user::{LocalUser, LocalUserUpdateForm};
use lemmy_db_views::structs::{LocalUserView, SiteView};
use lemmy_utils::error::{LemmyErrorType, LemmyResult};

/// Generate a new secret for two-factor-authentication. Afterwards you need to call [toggle_totp]
/// to enable it. This can only be called if 2FA is currently disabled.
#[tracing::instrument(skip(context))]
pub async fn generate_totp_secret(
  local_user_view: LocalUserView,
  context: Data<LemmyContext>,
) -> LemmyResult<Json<GenerateTotpSecretResponse>> {
  let site_view = SiteView::read_local(&mut context.pool()).await?;

  if local_user_view.local_user.totp_2fa_enabled {
    return Err(LemmyErrorType::TotpAlreadyEnabled)?;
  }

  let secret = generate_totp_2fa_secret();
  let secret_url =
    build_totp_2fa(&site_view.site.name, &local_user_view.person.name, &secret)?.get_url();

  let local_user_form = LocalUserUpdateForm {
    totp_2fa_secret: Some(Some(secret)),
    ..Default::default()
  };
  LocalUser::update(
    &mut context.pool(),
    local_user_view.local_user.id,
    &local_user_form,
  )
  .await?;

  Ok(Json(GenerateTotpSecretResponse {
    totp_secret_url: Sensitive::new(secret_url),
  }))
}
