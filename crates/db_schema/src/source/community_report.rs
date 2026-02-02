use crate::newtypes::{CommunityId, CommunityReportId};
use chrono::{DateTime, Utc};
use lemmy_db_schema_file::PersonId;
#[cfg(feature = "full")]
use lemmy_db_schema_file::schema::community_report;
use lemmy_diesel_utils::dburl::DbUrl;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

#[skip_serializing_none]
#[derive(PartialEq, Eq, Serialize, Deserialize, Debug, Clone)]
#[cfg_attr(
  feature = "full",
  derive(Queryable, Selectable, Associations, Identifiable)
)]
#[cfg_attr(
  feature = "full",
  diesel(belongs_to(crate::source::community::Community))
)]
#[cfg_attr(feature = "full", diesel(table_name = community_report))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// A comment report.
pub struct CommunityReport {
  pub id: CommunityReportId,
  pub creator_id: PersonId,
  pub community_id: CommunityId,
  pub original_community_name: String,
  pub original_community_title: String,
  pub original_community_summary: Option<String>,
  pub original_community_sidebar: Option<String>,
  pub original_community_icon: Option<String>,
  pub original_community_banner: Option<String>,
  pub reason: String,
  pub resolved: bool,
  pub resolver_id: Option<PersonId>,
  pub published_at: DateTime<Utc>,
  pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Clone)]
#[cfg_attr(feature = "full", derive(Insertable, AsChangeset))]
#[cfg_attr(feature = "full", diesel(table_name = community_report))]
pub struct CommunityReportForm {
  pub creator_id: PersonId,
  pub community_id: CommunityId,
  pub original_community_name: String,
  pub original_community_title: String,
  pub original_community_summary: Option<String>,
  pub original_community_sidebar: Option<String>,
  pub original_community_icon: Option<DbUrl>,
  pub original_community_banner: Option<DbUrl>,
  pub reason: String,
}
