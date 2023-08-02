use crate::newtypes::{DbUrl, InstanceId, SiteId};
#[cfg(feature = "full")]
use crate::schema::site;
use chrono::{DateTime, NaiveDateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
#[cfg(feature = "full")]
use ts_rs::TS;
use typed_builder::TypedBuilder;

#[skip_serializing_none]
#[derive(PartialEq, Eq, Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "full", derive(Queryable, Identifiable, TS))]
#[cfg_attr(feature = "full", diesel(table_name = site))]
#[cfg_attr(feature = "full", ts(export))]
/// The site.
pub struct Site {
  pub id: SiteId,
  pub name: String,
  /// A sidebar for the site in markdown.
  pub sidebar: Option<String>,
  pub published: DateTime<Utc>,
  pub updated: Option<DateTime<Utc>>,
  /// An icon URL.
  pub icon: Option<DbUrl>,
  /// A banner url.
  pub banner: Option<DbUrl>,
  /// A shorter, one-line description of the site.
  pub description: Option<String>,
  /// The federated actor_id.
  pub actor_id: DbUrl,
  /// The time the site was last refreshed.
  pub last_refreshed_at: DateTime<Utc>,
  /// The site inbox
  pub inbox_url: DbUrl,
  pub private_key: Option<String>,
  pub public_key: String,
  pub instance_id: InstanceId,
}

#[derive(Clone, TypedBuilder)]
#[builder(field_defaults(default))]
#[cfg_attr(feature = "full", derive(Insertable, AsChangeset))]
#[cfg_attr(feature = "full", diesel(table_name = site))]
pub struct SiteInsertForm {
  #[builder(!default)]
  pub name: String,
  pub sidebar: Option<String>,
  pub updated: Option<DateTime<Utc>>,
  pub icon: Option<DbUrl>,
  pub banner: Option<DbUrl>,
  pub description: Option<String>,
  pub actor_id: Option<DbUrl>,
  pub last_refreshed_at: Option<DateTime<Utc>>,
  pub inbox_url: Option<DbUrl>,
  pub private_key: Option<String>,
  pub public_key: Option<String>,
  #[builder(!default)]
  pub instance_id: InstanceId,
}

#[derive(Clone, TypedBuilder)]
#[builder(field_defaults(default))]
#[cfg_attr(feature = "full", derive(AsChangeset))]
#[cfg_attr(feature = "full", diesel(table_name = site))]
pub struct SiteUpdateForm {
  pub name: Option<String>,
  pub sidebar: Option<Option<String>>,
  pub updated: Option<Option<DateTime<Utc>>>,
  // when you want to null out a column, you have to send Some(None)), since sending None means you just don't want to update that column.
  pub icon: Option<Option<DbUrl>>,
  pub banner: Option<Option<DbUrl>>,
  pub description: Option<Option<String>>,
  pub actor_id: Option<DbUrl>,
  pub last_refreshed_at: Option<DateTime<Utc>>,
  pub inbox_url: Option<DbUrl>,
  pub private_key: Option<Option<String>>,
  pub public_key: Option<String>,
}
