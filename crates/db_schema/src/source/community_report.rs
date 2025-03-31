use crate::newtypes::{CommunityId, CommunityReportId, DbUrl, PersonId};
use chrono::{DateTime, Utc};
#[cfg(feature = "full")]
use lemmy_db_schema_file::schema::community_report;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
#[cfg(feature = "full")]
use ts_rs::TS;

#[skip_serializing_none]
#[derive(PartialEq, Eq, Serialize, Deserialize, Debug, Clone)]
#[cfg_attr(
  feature = "full",
  derive(Queryable, Selectable, Associations, Identifiable, TS)
)]
#[cfg_attr(
  feature = "full",
  diesel(belongs_to(crate::source::community::Community))
)]
#[cfg_attr(feature = "full", diesel(table_name = community_report))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "full", ts(export))]
/// A comment report.
pub struct CommunityReport {
  pub id: CommunityReportId,
  pub creator_id: PersonId,
  pub community_id: CommunityId,
  pub original_community_name: String,
  pub original_community_title: String,
  #[cfg_attr(feature = "full", ts(optional))]
  pub original_community_description: Option<String>,
  #[cfg_attr(feature = "full", ts(optional))]
  pub original_community_sidebar: Option<String>,
  #[cfg_attr(feature = "full", ts(optional))]
  pub original_community_icon: Option<String>,
  #[cfg_attr(feature = "full", ts(optional))]
  pub original_community_banner: Option<String>,
  pub reason: String,
  pub resolved: bool,
  #[cfg_attr(feature = "full", ts(optional))]
  pub resolver_id: Option<PersonId>,
  pub published: DateTime<Utc>,
  #[cfg_attr(feature = "full", ts(optional))]
  pub updated: Option<DateTime<Utc>>,
}

#[derive(Clone)]
#[cfg_attr(feature = "full", derive(Insertable, AsChangeset))]
#[cfg_attr(feature = "full", diesel(table_name = community_report))]
pub struct CommunityReportForm {
  pub creator_id: PersonId,
  pub community_id: CommunityId,
  pub original_community_name: String,
  pub original_community_title: String,
  pub original_community_description: Option<String>,
  pub original_community_sidebar: Option<String>,
  pub original_community_icon: Option<DbUrl>,
  pub original_community_banner: Option<DbUrl>,
  pub reason: String,
}
