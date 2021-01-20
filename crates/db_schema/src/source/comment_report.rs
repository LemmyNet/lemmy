use crate::{schema::comment_report, source::comment::Comment};
use serde::{Deserialize, Serialize};

#[derive(
  Identifiable, Queryable, Associations, PartialEq, Serialize, Deserialize, Debug, Clone,
)]
#[belongs_to(Comment)]
#[table_name = "comment_report"]
pub struct CommentReport {
  pub id: i32,
  pub creator_id: i32,
  pub comment_id: i32,
  pub original_comment_text: String,
  pub reason: String,
  pub resolved: bool,
  pub resolver_id: Option<i32>,
  pub published: chrono::NaiveDateTime,
  pub updated: Option<chrono::NaiveDateTime>,
}

#[derive(Insertable, AsChangeset, Clone)]
#[table_name = "comment_report"]
pub struct CommentReportForm {
  pub creator_id: i32,
  pub comment_id: i32,
  pub original_comment_text: String,
  pub reason: String,
}
