use lemmy_db_schema::newtypes::ExternalAuthId;
use lemmy_db_views::structs::ExternalAuthView;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
#[cfg(feature = "full")]
use ts_rs::TS;
use url::Url;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
/// Create an external auth method.
pub struct CreateExternalAuth {
  pub display_name: String,
  pub auth_type: String,
  #[cfg_attr(feature = "full", ts(type = "string"))]
  pub auth_endpoint: Url,
  #[cfg_attr(feature = "full", ts(type = "string"))]
  pub token_endpoint: Url,
  #[cfg_attr(feature = "full", ts(type = "string"))]
  pub user_endpoint: Url,
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
  #[cfg_attr(feature = "full", ts(type = "string"))]
  pub auth_endpoint: Url,
  #[cfg_attr(feature = "full", ts(type = "string"))]
  pub token_endpoint: Url,
  #[cfg_attr(feature = "full", ts(type = "string"))]
  pub user_endpoint: Url,
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

#[derive(Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
/// The response for deleting an external auth method.
pub struct DeleteExternalAuthResponse {
  pub id: ExternalAuthId,
  pub success: bool,
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
