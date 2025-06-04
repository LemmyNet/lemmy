use lemmy_db_schema::newtypes::OAuthProviderId;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
#[cfg(feature = "full")]
use ts_rs::TS;

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
/// Edit an external auth method.
pub struct EditOAuthProvider {
  pub id: OAuthProviderId,
  #[cfg_attr(feature = "full", ts(optional))]
  pub display_name: Option<String>,
  #[cfg_attr(feature = "full", ts(optional))]
  pub authorization_endpoint: Option<String>,
  #[cfg_attr(feature = "full", ts(optional))]
  pub token_endpoint: Option<String>,
  #[cfg_attr(feature = "full", ts(optional))]
  pub userinfo_endpoint: Option<String>,
  #[cfg_attr(feature = "full", ts(optional))]
  pub id_claim: Option<String>,
  #[cfg_attr(feature = "full", ts(optional))]
  pub client_secret: Option<String>,
  #[cfg_attr(feature = "full", ts(optional))]
  pub scopes: Option<String>,
  #[cfg_attr(feature = "full", ts(optional))]
  pub auto_verify_email: Option<bool>,
  #[cfg_attr(feature = "full", ts(optional))]
  pub account_linking_enabled: Option<bool>,
  #[cfg_attr(feature = "full", ts(optional))]
  pub use_pkce: Option<bool>,
  #[cfg_attr(feature = "full", ts(optional))]
  pub enabled: Option<bool>,
}
