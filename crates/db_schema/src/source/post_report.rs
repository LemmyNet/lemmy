use crate::newtypes::{DbUrl, PersonId, PostId, PostReportId};
#[cfg(feature = "full")]
use crate::schema::post_report;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
#[cfg(feature = "full")]
use ts_rs::TS;

#[skip_serializing_none]
#[derive(PartialEq, Eq, Serialize, Deserialize, Debug, Clone)]
#[cfg_attr(
  feature = "full",
  derive(Identifiable, Queryable, Selectable, Associations, TS)
)]
#[cfg_attr(feature = "full", diesel(belongs_to(crate::source::post::Post)))] // Is this the right assoc?
#[cfg_attr(feature = "full", diesel(table_name = post_report))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "full", ts(export))]
/// A post report.
pub struct PostReport {
  pub id: PostReportId,
  pub creator_id: PersonId,
  pub post_id: PostId,
  /// The original post title.
  pub original_post_name: String,
  /// The original post url.
  #[cfg_attr(feature = "full", ts(optional))]
  pub original_post_url: Option<DbUrl>,
  /// The original post body.
  #[cfg_attr(feature = "full", ts(optional))]
  pub original_post_body: Option<String>,
  pub reason: String,
  pub resolved: bool,
  #[cfg_attr(feature = "full", ts(optional))]
  pub resolver_id: Option<PersonId>,
  pub published: DateTime<Utc>,
  #[cfg_attr(feature = "full", ts(optional))]
  pub updated: Option<DateTime<Utc>>,
  pub violates_instance_rules: bool,
}

#[derive(Clone, Default)]
#[cfg_attr(feature = "full", derive(Insertable, AsChangeset))]
#[cfg_attr(feature = "full", diesel(table_name = post_report))]
pub struct PostReportForm {
  pub creator_id: PersonId,
  pub post_id: PostId,
  pub original_post_name: String,
  pub original_post_url: Option<DbUrl>,
  pub original_post_body: Option<String>,
  pub reason: String,
  pub violates_instance_rules: bool,
}
