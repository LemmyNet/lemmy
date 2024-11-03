use lemmy_db_schema::newtypes::OAuthProviderId;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
#[cfg(feature = "full")]
use ts_rs::TS;
use url::Url;

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
/// Create an external auth method.
pub struct CreateOAuthProvider {
  pub display_name: String,
  pub issuer: String,
  pub authorization_endpoint: String,
  pub token_endpoint: String,
  pub userinfo_endpoint: String,
  pub id_claim: String,
  pub client_id: String,
  pub client_secret: String,
  pub scopes: String,
  #[ts(optional)]
  pub auto_verify_email: Option<bool>,
  #[ts(optional)]
  pub account_linking_enabled: Option<bool>,
  #[ts(optional)]
  pub enabled: Option<bool>,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
/// Edit an external auth method.
pub struct EditOAuthProvider {
  pub id: OAuthProviderId,
  #[ts(optional)]
  pub display_name: Option<String>,
  #[ts(optional)]
  pub authorization_endpoint: Option<String>,
  #[ts(optional)]
  pub token_endpoint: Option<String>,
  #[ts(optional)]
  pub userinfo_endpoint: Option<String>,
  #[ts(optional)]
  pub id_claim: Option<String>,
  #[ts(optional)]
  pub client_secret: Option<String>,
  #[ts(optional)]
  pub scopes: Option<String>,
  #[ts(optional)]
  pub auto_verify_email: Option<bool>,
  #[ts(optional)]
  pub account_linking_enabled: Option<bool>,
  #[ts(optional)]
  pub enabled: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
/// Delete an external auth method.
pub struct DeleteOAuthProvider {
  pub id: OAuthProviderId,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
/// Logging in with an OAuth 2.0 authorization
pub struct AuthenticateWithOauth {
  pub code: String,
  pub oauth_provider_id: OAuthProviderId,
  pub redirect_uri: Url,
  #[ts(optional)]
  pub show_nsfw: Option<bool>,
  /// Username is mandatory at registration time
  #[ts(optional)]
  pub username: Option<String>,
  /// An answer is mandatory if require application is enabled on the server
  #[ts(optional)]
  pub answer: Option<String>,
}
