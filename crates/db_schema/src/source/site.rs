use crate::newtypes::{DbUrl, InstanceId, SiteId};
#[cfg(feature = "full")]
use crate::schema::site;
use serde::{Deserialize, Serialize};
use typed_builder::TypedBuilder;

#[derive(PartialEq, Eq, Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "full", derive(Queryable, Identifiable))]
#[cfg_attr(feature = "full", diesel(table_name = site))]
pub struct Site {
  pub id: SiteId,
  pub name: String,
  pub sidebar: Option<String>,
  pub published: chrono::NaiveDateTime,
  pub updated: Option<chrono::NaiveDateTime>,
  pub icon: Option<DbUrl>,
  pub banner: Option<DbUrl>,
  pub description: Option<String>,
  pub actor_id: DbUrl,
  pub last_refreshed_at: chrono::NaiveDateTime,
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
  pub updated: Option<chrono::NaiveDateTime>,
  pub icon: Option<DbUrl>,
  pub banner: Option<DbUrl>,
  pub description: Option<String>,
  pub actor_id: Option<DbUrl>,
  pub last_refreshed_at: Option<chrono::NaiveDateTime>,
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
  pub updated: Option<Option<chrono::NaiveDateTime>>,
  // when you want to null out a column, you have to send Some(None)), since sending None means you just don't want to update that column.
  pub icon: Option<Option<DbUrl>>,
  pub banner: Option<Option<DbUrl>>,
  pub description: Option<Option<String>>,
  pub actor_id: Option<DbUrl>,
  pub last_refreshed_at: Option<chrono::NaiveDateTime>,
  pub inbox_url: Option<DbUrl>,
  pub private_key: Option<Option<String>>,
  pub public_key: Option<String>,
}
