use crate::newtypes::{LocalUserId, OAuthProviderId};
use chrono::{DateTime, Utc};
#[cfg(feature = "full")]
use lemmy_db_schema_file::schema::oauth_account;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

#[skip_serializing_none]
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "full", derive(Queryable, Selectable))]
#[cfg_attr(feature = "full", diesel(table_name = oauth_account))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// An auth account method.
pub struct OAuthAccount {
  pub local_user_id: LocalUserId,
  pub oauth_provider_id: OAuthProviderId,
  pub oauth_user_id: String,
  pub published_at: DateTime<Utc>,
  pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, derive_new::new)]
#[cfg_attr(feature = "full", derive(Insertable, AsChangeset))]
#[cfg_attr(feature = "full", diesel(table_name = oauth_account))]
pub struct OAuthAccountInsertForm {
  pub local_user_id: LocalUserId,
  pub oauth_provider_id: OAuthProviderId,
  pub oauth_user_id: String,
}
