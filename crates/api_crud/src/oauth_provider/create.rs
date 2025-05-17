use activitypub_federation::config::Data;
use actix_web::web::Json;
use lemmy_api_common::{
  context::LemmyContext,
  oauth_provider::CreateOAuthProvider,
  utils::is_admin,
};
use lemmy_db_schema::{
  source::oauth_provider::{OAuthProvider, OAuthProviderInsertForm},
  traits::Crud,
};
use lemmy_db_views_local_user::LocalUserView;
use lemmy_utils::error::LemmyError;
use url::Url;

pub async fn create_oauth_provider(
  data: Json<CreateOAuthProvider>,
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> Result<Json<OAuthProvider>, LemmyError> {
  // Make sure user is an admin
  is_admin(&local_user_view)?;

  let cloned_data = data.clone();
  let oauth_provider_form = OAuthProviderInsertForm {
    display_name: cloned_data.display_name,
    issuer: Url::parse(&cloned_data.issuer)?.into(),
    authorization_endpoint: Url::parse(&cloned_data.authorization_endpoint)?.into(),
    token_endpoint: Url::parse(&cloned_data.token_endpoint)?.into(),
    userinfo_endpoint: Url::parse(&cloned_data.userinfo_endpoint)?.into(),
    id_claim: cloned_data.id_claim,
    client_id: data.client_id.to_string(),
    client_secret: data.client_secret.to_string(),
    scopes: data.scopes.to_string(),
    auto_verify_email: data.auto_verify_email,
    account_linking_enabled: data.account_linking_enabled,
    use_pkce: data.use_pkce,
    enabled: data.enabled,
  };
  let oauth_provider = OAuthProvider::create(&mut context.pool(), &oauth_provider_form).await?;
  Ok(Json(oauth_provider))
}
