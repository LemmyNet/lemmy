use crate::{
  newtypes::{DbUrl, OAuthProviderId},
  sensitive::SensitiveString,
};
use chrono::{DateTime, Utc};
#[cfg(feature = "full")]
use lemmy_db_schema_file::schema::oauth_provider;
use serde::{
  Deserialize,
  Serialize,
  ser::{SerializeStruct, Serializer},
};
use serde_with::skip_serializing_none;

#[skip_serializing_none]
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "full", derive(Queryable, Selectable, Identifiable))]
#[cfg_attr(feature = "full", diesel(table_name = oauth_provider))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// oauth provider with client_secret - should never be sent to the client
pub struct OAuthProvider {
  pub id: OAuthProviderId,
  /// The OAuth 2.0 provider name displayed to the user on the Login page
  pub display_name: String,
  /// The issuer url of the OAUTH provider.
  #[cfg_attr(feature = "ts-rs", ts(type = "string"))]
  pub issuer: DbUrl,
  /// The authorization endpoint is used to interact with the resource owner and obtain an
  /// authorization grant. This is usually provided by the OAUTH provider.
  #[cfg_attr(feature = "ts-rs", ts(type = "string"))]
  pub authorization_endpoint: DbUrl,
  /// The token endpoint is used by the client to obtain an access token by presenting its
  /// authorization grant or refresh token. This is usually provided by the OAUTH provider.
  #[cfg_attr(feature = "ts-rs", ts(type = "string"))]
  pub token_endpoint: DbUrl,
  /// The UserInfo Endpoint is an OAuth 2.0 Protected Resource that returns Claims about the
  /// authenticated End-User. This is defined in the OIDC specification.
  #[cfg_attr(feature = "ts-rs", ts(type = "string"))]
  pub userinfo_endpoint: DbUrl,
  /// The OAuth 2.0 claim containing the unique user ID returned by the provider. Usually this
  /// should be set to "sub".
  pub id_claim: String,
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
  pub published_at: DateTime<Utc>,
  pub updated_at: Option<DateTime<Utc>>,
  /// switch to enable or disable PKCE
  pub use_pkce: bool,
}

#[derive(Clone, PartialEq, Eq, Debug, Deserialize)]
#[serde(transparent)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
// A subset of OAuthProvider used for public requests, for example to display the OAUTH buttons on
// the login page
pub struct PublicOAuthProvider(pub OAuthProvider);

impl Serialize for PublicOAuthProvider {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where
    S: Serializer,
  {
    let mut state = serializer.serialize_struct("PublicOAuthProvider", 5)?;
    state.serialize_field("id", &self.0.id)?;
    state.serialize_field("display_name", &self.0.display_name)?;
    state.serialize_field("authorization_endpoint", &self.0.authorization_endpoint)?;
    state.serialize_field("client_id", &self.0.client_id)?;
    state.serialize_field("scopes", &self.0.scopes)?;
    state.serialize_field("use_pkce", &self.0.use_pkce)?;
    state.end()
  }
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "full", derive(Insertable, AsChangeset))]
#[cfg_attr(feature = "full", diesel(table_name = oauth_provider))]
pub struct OAuthProviderInsertForm {
  pub display_name: String,
  pub issuer: DbUrl,
  pub authorization_endpoint: DbUrl,
  pub token_endpoint: DbUrl,
  pub userinfo_endpoint: DbUrl,
  pub id_claim: String,
  pub client_id: String,
  pub client_secret: String,
  pub scopes: String,
  pub auto_verify_email: Option<bool>,
  pub account_linking_enabled: Option<bool>,
  pub use_pkce: Option<bool>,
  pub enabled: Option<bool>,
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "full", derive(Insertable, AsChangeset))]
#[cfg_attr(feature = "full", diesel(table_name = oauth_provider))]
pub struct OAuthProviderUpdateForm {
  pub display_name: Option<String>,
  pub authorization_endpoint: Option<DbUrl>,
  pub token_endpoint: Option<DbUrl>,
  pub userinfo_endpoint: Option<DbUrl>,
  pub id_claim: Option<String>,
  pub client_secret: Option<String>,
  pub scopes: Option<String>,
  pub auto_verify_email: Option<bool>,
  pub account_linking_enabled: Option<bool>,
  pub use_pkce: Option<bool>,
  pub enabled: Option<bool>,
  pub updated_at: Option<Option<DateTime<Utc>>>,
}
