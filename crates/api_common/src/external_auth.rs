use lemmy_db_schema::newtypes::ExternalAuthId;
use lemmy_db_views::structs::ExternalAuthView;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
#[cfg(feature = "full")]
use ts_rs::TS;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
/// Create an external auth method.
pub struct CreateExternalAuth {
  pub display_name: String,
  pub auth_type: String,
  pub auth_endpoint: String,
  pub token_endpoint: String,
  pub user_endpoint: String,
  pub id_attribute: String,
  pub issuer: String,
  pub client_id: String,
  pub client_secret: String,
  pub scopes: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
/// Edit an external auth method.
pub struct EditExternalAuth {
  pub id: ExternalAuthId,
  pub display_name: String,
  pub auth_type: String,
  pub auth_endpoint: String,
  pub token_endpoint: String,
  pub user_endpoint: String,
  pub id_attribute: String,
  pub issuer: String,
  pub client_id: String,
  pub client_secret: String,
  pub scopes: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
/// Delete an external auth method.
pub struct DeleteExternalAuth {
  pub id: ExternalAuthId,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
/// A response for an external auth method.
pub struct ExternalAuthResponse {
  pub external_auth: ExternalAuthView,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
/// Logging in with an OAuth 2.0 token
pub struct OAuth {
  pub code: String,
  pub state: String,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
/// Response from OAuth token endpoint
pub struct TokenResponse {
  pub access_token: String,
  pub token_type: String,
  pub expires_in: Option<i64>,
  pub refresh_token: Option<String>,
  pub scope: Option<String>,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
/// State parameter from the auth endpoint response
pub struct OAuthResponse {
  pub external_auth: i32,
  pub client_redirect_uri: String,
}
