use crate::newtypes::{CommunityId, CommunityReportId, PersonId};
#[cfg(feature = "full")]
use crate::schema::community_report;
use chrono::{DateTime, Utc};
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
  pub original_community_title: String,
  pub original_community_description: String,
  pub original_community_icon: String,
  pub original_community_banner: String,
  pub reason: String,
  pub resolved: bool,
  pub resolver_id: Option<PersonId>,
  pub published: DateTime<Utc>,
  pub updated: Option<DateTime<Utc>>,
}

#[derive(Clone)]
#[cfg_attr(feature = "full", derive(Insertable, AsChangeset))]
#[cfg_attr(feature = "full", diesel(table_name = community_report))]
pub struct CommunityReportForm {
  pub creator_id: PersonId,
  pub community_id: CommunityId,
  pub original_community_title: String,
  pub original_community_description: String,
  pub original_community_icon: String,
  pub original_community_banner: String,
  pub reason: String,
}
