use crate::newtypes::ExternalAuthId;
#[cfg(feature = "full")]
use crate::schema::external_auth;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
#[cfg(feature = "full")]
use ts_rs::TS;
use typed_builder::TypedBuilder;

#[skip_serializing_none]
#[derive(PartialEq, Eq, Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "full", derive(Queryable, Identifiable, TS))]
#[cfg_attr(feature = "full", diesel(table_name = external_auth))]
#[cfg_attr(feature = "full", ts(export))]
/// An external auth method.
pub struct ExternalAuth {
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
  pub published: DateTime<Utc>,
  pub updated: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, TypedBuilder)]
#[cfg_attr(feature = "full", derive(Insertable, AsChangeset))]
#[cfg_attr(feature = "full", diesel(table_name = external_auth))]
pub struct ExternalAuthInsertForm {
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

#[derive(Debug, Clone, TypedBuilder)]
#[cfg_attr(feature = "full", derive(Insertable, AsChangeset))]
#[cfg_attr(feature = "full", diesel(table_name = external_auth))]
pub struct ExternalAuthUpdateForm {
  pub display_name: String,
  pub auth_type: String,
  pub auth_endpoint: String,
  pub token_endpoint: String,
  pub user_endpoint: String,
  pub id_attribute: String,
  pub issuer: String,
  pub client_id: String,
  pub client_secret: Option<String>,
  pub scopes: String,
}
