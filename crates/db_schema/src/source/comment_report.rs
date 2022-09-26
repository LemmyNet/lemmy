use crate::newtypes::{CommentId, CommentReportId, PersonId};
use serde::{Deserialize, Serialize};

#[cfg(feature = "full")]
use crate::schema::comment_report;

#[derive(PartialEq, Eq, Serialize, Deserialize, Debug, Clone)]
#[cfg_attr(feature = "full", derive(Queryable, Associations, Identifiable))]
#[cfg_attr(feature = "full", diesel(belongs_to(crate::source::comment::Comment)))]
#[cfg_attr(feature = "full", diesel(table_name = comment_report))]
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
