use crate::newtypes::LocalSiteId;
#[cfg(feature = "full")]
use crate::schema::tagline;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
#[cfg(feature = "full")]
use ts_rs::TS;

#[skip_serializing_none]
#[derive(PartialEq, Eq, Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "full", derive(Queryable, Associations, Identifiable, TS))]
#[cfg_attr(feature = "full", diesel(table_name = tagline))]
#[cfg_attr(
  feature = "full",
  diesel(belongs_to(crate::source::local_site::LocalSite))
)]
#[cfg_attr(feature = "full", ts(export))]
pub struct Tagline {
  pub id: i32,
  pub local_site_id: LocalSiteId,
  pub content: String,
  pub published: chrono::NaiveDateTime,
  pub updated: Option<chrono::NaiveDateTime>,
}

#[derive(Clone, Default)]
#[cfg_attr(feature = "full", derive(Insertable, AsChangeset))]
#[cfg_attr(feature = "full", diesel(table_name = tagline))]
pub struct TaglineForm {
  pub local_site_id: LocalSiteId,
  pub content: String,
  pub updated: Option<chrono::NaiveDateTime>,
}
