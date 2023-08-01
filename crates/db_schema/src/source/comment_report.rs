use crate::newtypes::{CommentId, CommentReportId, PersonId};
#[cfg(feature = "full")]
use crate::schema::comment_report;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
#[cfg(feature = "full")]
use ts_rs::TS;

#[skip_serializing_none]
#[derive(PartialEq, Eq, Serialize, Deserialize, Debug, Clone)]
#[cfg_attr(feature = "full", derive(Queryable, Associations, Identifiable, TS, WithoutId!))]
#[cfg_attr(feature = "full", diesel(belongs_to(crate::source::comment::Comment)))]
#[cfg_attr(feature = "full", diesel(table_name = comment_report))]
#[cfg_attr(feature = "full", ts(export))]
/// A comment report.
pub struct CommentReport {
  pub id: CommentReportId,
  pub creator_id: PersonId,
  pub comment_id: CommentId,
  pub original_comment_text: String,
  pub reason: String,
  pub resolved: bool,
  pub resolver_id: Option<PersonId>,
  pub published: chrono::NaiveDateTime,
  pub updated: Option<chrono::NaiveDateTime>,
}

#[derive(Clone)]
#[cfg_attr(feature = "full", derive(Insertable, AsChangeset))]
#[cfg_attr(feature = "full", diesel(table_name = comment_report))]
pub struct CommentReportForm {
  pub creator_id: PersonId,
  pub comment_id: CommentId,
  pub original_comment_text: String,
  pub reason: String,
}
