use activitypub_federation::config::Data;
use actix_web::web::Json;
use chrono::Utc;
use lemmy_api_utils::{context::LemmyContext, utils::is_admin};
use lemmy_db_schema::source::oauth_provider::{AdminOAuthProvider, OAuthProviderUpdateForm};
use lemmy_db_views_local_user::LocalUserView;
use lemmy_db_views_site::api::EditOAuthProvider;
use lemmy_diesel_utils::{
  traits::Crud,
  utils::{diesel_required_string_update, diesel_required_url_update},
};
use lemmy_utils::error::LemmyError;

pub async fn edit_oauth_provider(
  Json(data): Json<EditOAuthProvider>,
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> Result<Json<AdminOAuthProvider>, LemmyError> {
  // Make sure user is an admin
  is_admin(&local_user_view)?;

  let cloned_data = data.clone();
  let oauth_provider_form = OAuthProviderUpdateForm {
    display_name: diesel_required_string_update(cloned_data.display_name.as_deref()),
    authorization_endpoint: diesel_required_url_update(
      cloned_data.authorization_endpoint.as_deref(),
    )?,
    token_endpoint: diesel_required_url_update(cloned_data.token_endpoint.as_deref())?,
    userinfo_endpoint: diesel_required_url_update(cloned_data.userinfo_endpoint.as_deref())?,
    id_claim: diesel_required_string_update(data.id_claim.as_deref()),
    client_secret: diesel_required_string_update(data.client_secret.as_deref()),
    scopes: diesel_required_string_update(data.scopes.as_deref()),
    auto_verify_email: data.auto_verify_email,
    account_linking_enabled: data.account_linking_enabled,
    enabled: data.enabled,
    use_pkce: data.use_pkce,
    updated_at: Some(Some(Utc::now())),
  };

  let update_result =
    AdminOAuthProvider::update(&mut context.pool(), data.id, &oauth_provider_form).await?;
  let oauth_provider = AdminOAuthProvider::read(&mut context.pool(), update_result.id).await?;
  Ok(Json(oauth_provider))
}
