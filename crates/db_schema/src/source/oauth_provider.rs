#[cfg(feature = "full")]
use crate::schema::oauth_provider;
use crate::{
  newtypes::{DbUrl, OAuthProviderId},
  sensitive::SensitiveString,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
#[cfg(feature = "full")]
use ts_rs::TS;

#[skip_serializing_none]
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "full", derive(Queryable, Selectable, Identifiable, TS))]
#[cfg_attr(feature = "full", diesel(table_name = oauth_provider))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
/// oauth provider with client_secret - should never be sent to the client
pub struct UnsafeOAuthProvider {
  pub id: OAuthProviderId,
  /// The OAuth 2.0 provider name displayed to the user on the Login page
  pub display_name: String,
  /// The issuer url of the OAUTH provider.
  #[cfg_attr(feature = "full", ts(type = "string"))]
  pub issuer: DbUrl,
  /// The authorization endpoint is used to interact with the resource owner and obtain an
  /// authorization grant. This is usually provided by the OAUTH provider.
  #[cfg_attr(feature = "full", ts(type = "string"))]
  pub authorization_endpoint: DbUrl,
  /// The token endpoint is used by the client to obtain an access token by presenting its
  /// authorization grant or refresh token. This is usually provided by the OAUTH provider.
  #[cfg_attr(feature = "full", ts(type = "string"))]
  pub token_endpoint: DbUrl,
  /// The UserInfo Endpoint is an OAuth 2.0 Protected Resource that returns Claims about the
  /// authenticated End-User. This is defined in the OIDC specification.
  #[cfg_attr(feature = "full", ts(type = "string"))]
  pub userinfo_endpoint: DbUrl,
  /// The OAuth 2.0 claim containing the unique user ID returned by the provider. Usually this
  /// should be set to "sub".
  pub id_claim: String,
  /// The OAuth 2.0 claim containing the user name returned by the provider. This depends on the
  /// provider and could be "name", "username", ...
  pub name_claim: String,
  /// The client_id is provided by the OAuth 2.0 provider and is a unique identifier to this
  /// service
  pub client_id: String,
  /// The client_secret is provided by the OAuth 2.0 provider and is used to authenticate this
  /// service with the provider
  #[serde(skip)]
  pub client_secret: SensitiveString,
  /// Lists the scopes requested from users. Users will have to grant access to the requested scope
  /// at sign up.
  pub scopes: String,
  /// Automatically sets email as verified on registration
  pub auto_verify_email: bool,
  /// Allows linking an OAUTH account to an existing user account by matching emails
  pub account_linking_enabled: bool,
  /// switch to enable or disable an oauth provider
  pub enabled: bool,
  pub published: DateTime<Utc>,
  pub updated: Option<DateTime<Utc>>,
}

#[skip_serializing_none]
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
pub struct OAuthProvider {
  pub id: OAuthProviderId,
  /// The OAuth 2.0 provider name displayed to the user on the Login page
  pub display_name: String,
  /// The issuer url of the OAUTH provider.
  #[cfg_attr(feature = "full", ts(type = "string"))]
  pub issuer: Option<DbUrl>,
  /// The authorization endpoint is used to interact with the resource owner and obtain an
  /// authorization grant. This is usually provided by the OAUTH provider.
  #[cfg_attr(feature = "full", ts(type = "string"))]
  pub authorization_endpoint: DbUrl,
  /// The token endpoint is used by the client to obtain an access token by presenting its
  /// authorization grant or refresh token. This is usually provided by the OAUTH provider.
  #[cfg_attr(feature = "full", ts(type = "string"))]
  pub token_endpoint: Option<DbUrl>,
  /// The UserInfo Endpoint is an OAuth 2.0 Protected Resource that returns Claims about the
  /// authenticated End-User. This is defined in the OIDC specification.
  #[cfg_attr(feature = "full", ts(type = "string"))]
  pub userinfo_endpoint: Option<DbUrl>,
  /// The OAuth 2.0 claim containing the unique user ID returned by the provider. Usually this
  /// should be set to "sub".
  pub id_claim: Option<String>,
  /// The OAuth 2.0 claim containing the user name returned by the provider. This depends on the
  /// provider and could be "name", "username", ...
  pub name_claim: Option<String>,
  /// The client_id is provided by the OAuth 2.0 provider and is a unique identifier to this
  /// service
  pub client_id: String,
  /// Lists the scopes requested from users. Users will have to grant access to the requested scope
  /// at sign up.
  pub scopes: String,
  /// Automatically sets email as verified on registration
  pub auto_verify_email: Option<bool>,
  /// Allows linking an OAUTH account to an existing user account by matching emails
  pub account_linking_enabled: Option<bool>,
  /// switch to enable or disable an oauth provider
  pub enabled: Option<bool>,
  pub published: Option<DateTime<Utc>>,
  pub updated: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "full", derive(Insertable, AsChangeset, TS))]
#[cfg_attr(feature = "full", diesel(table_name = oauth_provider))]
#[cfg_attr(feature = "full", ts(export))]
pub struct OAuthProviderInsertForm {
  pub display_name: String,
  #[cfg_attr(feature = "full", ts(type = "string"))]
  pub issuer: DbUrl,
  #[cfg_attr(feature = "full", ts(type = "string"))]
  pub authorization_endpoint: DbUrl,
  #[cfg_attr(feature = "full", ts(type = "string"))]
  pub token_endpoint: DbUrl,
  #[cfg_attr(feature = "full", ts(type = "string"))]
  pub userinfo_endpoint: DbUrl,
  pub id_claim: String,
  pub name_claim: String,
  pub client_id: String,
  pub client_secret: String,
  pub scopes: String,
  pub auto_verify_email: bool,
  pub account_linking_enabled: bool,
  pub enabled: bool,
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "full", derive(Insertable, AsChangeset, TS))]
#[cfg_attr(feature = "full", diesel(table_name = oauth_provider))]
#[cfg_attr(feature = "full", ts(export))]
pub struct OAuthProviderUpdateForm {
  pub display_name: Option<String>,
  #[cfg_attr(feature = "full", ts(type = "string"))]
  pub authorization_endpoint: Option<DbUrl>,
  #[cfg_attr(feature = "full", ts(type = "string"))]
  pub token_endpoint: Option<DbUrl>,
  #[cfg_attr(feature = "full", ts(type = "string"))]
  pub userinfo_endpoint: Option<DbUrl>,
  pub id_claim: Option<String>,
  pub name_claim: Option<String>,
  pub client_secret: Option<String>,
  pub scopes: Option<String>,
  pub auto_verify_email: Option<bool>,
  pub account_linking_enabled: Option<bool>,
  pub enabled: Option<bool>,
  pub updated: DateTime<Utc>,
}
