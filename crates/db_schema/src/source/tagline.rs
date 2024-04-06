use crate::newtypes::LocalSiteId;
#[cfg(feature = "full")]
use crate::schema::tagline;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
#[cfg(feature = "full")]
use ts_rs::TS;

#[skip_serializing_none]
#[derive(PartialEq, Eq, Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(
  feature = "full",
  derive(Queryable, Selectable, Associations, Identifiable, TS)
)]
#[cfg_attr(feature = "full", diesel(table_name = tagline))]
#[cfg_attr(
  feature = "full",
  diesel(belongs_to(crate::source::local_site::LocalSite))
)]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "full", ts(export))]
/// A tagline, shown at the top of your site.
pub struct Tagline {
  pub id: i32,
  pub local_site_id: LocalSiteId,
  pub content: String,
  pub published: DateTime<Utc>,
  pub updated: Option<DateTime<Utc>>,
}

#[derive(Clone, Default)]
#[cfg_attr(feature = "full", derive(Insertable, AsChangeset))]
#[cfg_attr(feature = "full", diesel(table_name = tagline))]
pub struct TaglineInsertForm {
  pub local_site_id: LocalSiteId,
  pub content: String,
}

#[derive(Clone, Default)]
#[cfg_attr(feature = "full", derive(Insertable, AsChangeset))]
#[cfg_attr(feature = "full", diesel(table_name = tagline))]
pub struct TaglineUpdateForm {
  pub content: String,
  pub updated: Option<Option<DateTime<Utc>>>,
}
