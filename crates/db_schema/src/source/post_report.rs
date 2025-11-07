use crate::newtypes::{PersonId, PostId, PostReportId};
use chrono::{DateTime, Utc};
#[cfg(feature = "full")]
use lemmy_db_schema_file::schema::post_report;
use lemmy_diesel_utils::dburl::DbUrl;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

#[skip_serializing_none]
#[derive(PartialEq, Eq, Serialize, Deserialize, Debug, Clone)]
#[cfg_attr(
  feature = "full",
  derive(Identifiable, Queryable, Selectable, Associations)
)]
#[cfg_attr(feature = "full", diesel(belongs_to(crate::source::post::Post)))] // Is this the right assoc?
#[cfg_attr(feature = "full", diesel(table_name = post_report))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// A post report.
pub struct PostReport {
  pub id: PostReportId,
  pub creator_id: PersonId,
  pub post_id: PostId,
  /// The original post title.
  pub original_post_name: String,
  /// The original post url.
  pub original_post_url: Option<DbUrl>,
  /// The original post body.
  pub original_post_body: Option<String>,
  pub reason: String,
  pub resolved: bool,
  pub resolver_id: Option<PersonId>,
  pub published_at: DateTime<Utc>,
  pub updated_at: Option<DateTime<Utc>>,
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
