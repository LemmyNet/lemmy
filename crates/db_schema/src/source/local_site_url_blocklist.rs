#[cfg(feature = "full")]
use crate::schema::local_site_url_blocklist;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
#[cfg(feature = "full")]
use ts_rs::TS;

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "full", derive(Queryable, Selectable, Identifiable, TS))]
#[cfg_attr(feature = "full", diesel(table_name = local_site_url_blocklist))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "full", ts(export))]
pub struct LocalSiteUrlBlocklist {
  pub id: i32,
  pub url: String,
  pub published: DateTime<Utc>,
  pub updated: Option<DateTime<Utc>>,
}

#[derive(Default, Clone)]
#[cfg_attr(feature = "full", derive(Insertable, AsChangeset))]
#[cfg_attr(feature = "full", diesel(table_name = local_site_url_blocklist))]
pub struct LocalSiteUrlBlocklistForm {
  pub url: String,
  pub updated: Option<DateTime<Utc>>,
}
