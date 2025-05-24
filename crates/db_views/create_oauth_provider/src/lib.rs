use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
#[cfg(feature = "full")]
use ts_rs::TS;

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
  #[cfg_attr(feature = "full", ts(optional))]
  pub auto_verify_email: Option<bool>,
  #[cfg_attr(feature = "full", ts(optional))]
  pub account_linking_enabled: Option<bool>,
  #[cfg_attr(feature = "full", ts(optional))]
  pub use_pkce: Option<bool>,
  #[cfg_attr(feature = "full", ts(optional))]
  pub enabled: Option<bool>,
}
