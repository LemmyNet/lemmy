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
/// Logging in with an OAuth 2.0 authorization
pub struct AuthenticateWithOauth {
  pub code: String,
  pub oauth_provider_id: OAuthProviderId,
  pub redirect_uri: Url,
  #[cfg_attr(feature = "full", ts(optional))]
  pub show_nsfw: Option<bool>,
  /// Username is mandatory at registration time
  #[cfg_attr(feature = "full", ts(optional))]
  pub username: Option<String>,
  /// An answer is mandatory if require application is enabled on the server
  #[cfg_attr(feature = "full", ts(optional))]
  pub answer: Option<String>,
  #[cfg_attr(feature = "full", ts(optional))]
  pub pkce_code_verifier: Option<String>,
}
