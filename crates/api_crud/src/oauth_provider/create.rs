use activitypub_federation::config::Data;
use actix_web::web::Json;
use lemmy_api_common::{
  context::LemmyContext,
  oauth_provider::CreateOAuthProvider,
  utils::is_admin,
};
use lemmy_db_schema::{
  newtypes::OAuthProviderId,
  source::oauth_provider::{OAuthProvider, OAuthProviderInsertForm, UnsafeOAuthProvider},
  traits::Crud,
};
use lemmy_db_views::structs::LocalUserView;
use lemmy_utils::error::LemmyError;
use sha3::{
  digest::{ExtendableOutput, Update, XofReader},
  Shake128,
};
use url::Url;

#[tracing::instrument(skip(context))]
pub async fn create_oauth_provider(
  data: Json<CreateOAuthProvider>,
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> Result<Json<OAuthProvider>, LemmyError> {
  // Make sure user is an admin
  is_admin(&local_user_view)?;

  // hash the issuer and client_id to create a deterministic i64 id
  let mut hasher = Shake128::default();
  hasher.update(data.issuer.as_bytes());
  hasher.update(data.client_id.as_bytes());
  let mut reader = hasher.finalize_xof();
  let mut id_bytes = [0u8; 8];
  reader.read(&mut id_bytes);

  let cloned_data = data.clone();
  let oauth_provider_form = OAuthProviderInsertForm::builder()
    .id(OAuthProviderId(i64::from_ne_bytes(id_bytes)))
    .display_name(cloned_data.display_name)
    .issuer(Url::parse(&cloned_data.issuer)?.into())
    .authorization_endpoint(Url::parse(&cloned_data.authorization_endpoint)?.into())
    .token_endpoint(Url::parse(&cloned_data.token_endpoint)?.into())
    .userinfo_endpoint(Url::parse(&cloned_data.userinfo_endpoint)?.into())
    .id_claim(cloned_data.id_claim)
    .name_claim(cloned_data.name_claim)
    .client_id(data.client_id.to_string())
    .client_secret(data.client_secret.to_string())
    .scopes(data.scopes.to_string())
    .auto_verify_email(data.auto_verify_email)
    .auto_approve_application(data.auto_approve_application)
    .account_linking_enabled(data.account_linking_enabled)
    .enabled(data.enabled)
    .build();
  let unsafe_oauth_provider =
    UnsafeOAuthProvider::create(&mut context.pool(), &oauth_provider_form).await?;
  Ok(Json(OAuthProvider::from_unsafe(&unsafe_oauth_provider)))
}
