use crate::{schema::comment_report, source::comment::Comment, CommentId, PersonId};
use serde::{Deserialize, Serialize};

#[derive(
  Identifiable, Queryable, Associations, PartialEq, Serialize, Deserialize, Debug, Clone,
)]
#[belongs_to(Comment)]
#[table_name = "comment_report"]
pub struct CommentReport {
  pub id: i32,
  pub creator_id: PersonId,
  pub comment_id: CommentId,
  pub original_comment_text: String,
  pub reason: String,
  pub resolved: bool,
  pub resolver_id: Option<PersonId>,
  pub published: chrono::NaiveDateTime,
  pub updated: Option<chrono::NaiveDateTime>,
}

#[derive(Insertable, AsChangeset, Clone)]
#[table_name = "comment_report"]
pub struct CommentReportForm {
  pub creator_id: PersonId,
  pub comment_id: CommentId,
  pub original_comment_text: String,
  pub reason: String,
}
