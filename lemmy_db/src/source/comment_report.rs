use diesel::{dsl::*, result::Error, *};
use serde::{Deserialize, Serialize};

use crate::{naive_now, schema::comment_report, source::comment::Comment, Reportable};

#[derive(Identifiable, Queryable, Associations, PartialEq, Serialize, Deserialize, Debug, Clone)]
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

impl Reportable<CommentReportForm> for CommentReport {
  /// creates a comment report and returns it
  ///
  /// * `conn` - the postgres connection
  /// * `comment_report_form` - the filled CommentReportForm to insert
  fn report(conn: &PgConnection, comment_report_form: &CommentReportForm) -> Result<Self, Error> {
    use crate::schema::comment_report::dsl::*;
    insert_into(comment_report)
      .values(comment_report_form)
      .get_result::<Self>(conn)
  }

  /// resolve a comment report
  ///
  /// * `conn` - the postgres connection
  /// * `report_id` - the id of the report to resolve
  /// * `by_resolver_id` - the id of the user resolving the report
  fn resolve(conn: &PgConnection, report_id: i32, by_resolver_id: i32) -> Result<usize, Error> {
    use crate::schema::comment_report::dsl::*;
    update(comment_report.find(report_id))
      .set((
        resolved.eq(true),
        resolver_id.eq(by_resolver_id),
        updated.eq(naive_now()),
      ))
      .execute(conn)
  }

  /// unresolve a comment report
  ///
  /// * `conn` - the postgres connection
  /// * `report_id` - the id of the report to unresolve
  /// * `by_resolver_id` - the id of the user unresolving the report
  fn unresolve(conn: &PgConnection, report_id: i32, by_resolver_id: i32) -> Result<usize, Error> {
    use crate::schema::comment_report::dsl::*;
    update(comment_report.find(report_id))
      .set((
        resolved.eq(false),
        resolver_id.eq(by_resolver_id),
        updated.eq(naive_now()),
      ))
      .execute(conn)
  }
}
