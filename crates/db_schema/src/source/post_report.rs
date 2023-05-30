use crate::newtypes::{DbUrl, PersonId, PostId, PostReportId};
#[cfg(feature = "full")]
use crate::schema::post_report;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
#[cfg(feature = "full")]
use ts_rs::TS;

#[skip_serializing_none]
#[derive(PartialEq, Eq, Serialize, Deserialize, Debug, Clone)]
#[cfg_attr(feature = "full", derive(Identifiable, Queryable, Associations, TS))]
#[cfg_attr(feature = "full", diesel(belongs_to(crate::source::post::Post)))] // Is this the right assoc?
#[cfg_attr(feature = "full", diesel(table_name = post_report))]
#[cfg_attr(feature = "full", ts(export))]
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
  pub published: chrono::NaiveDateTime,
  pub updated: Option<chrono::NaiveDateTime>,
}

#[derive(Clone)]
#[cfg_attr(feature = "full", derive(Insertable, AsChangeset))]
#[cfg_attr(feature = "full", diesel(table_name = post_report))]
pub struct PostReportForm {
  pub creator_id: PersonId,
  pub post_id: PostId,
  pub original_post_name: String,
  pub original_post_url: Option<DbUrl>,
  pub original_post_body: Option<String>,
  pub reason: String,
}
