use crate::{
  newtypes::{CommentReportId, PersonId},
  source::comment_report::{CommentReport, CommentReportForm},
  traits::Reportable,
  utils::naive_now,
};
use diesel::{dsl::*, result::Error, *};

impl Reportable for CommentReport {
  type Form = CommentReportForm;
  type IdType = CommentReportId;
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
  fn resolve(
    conn: &PgConnection,
    report_id: Self::IdType,
    by_resolver_id: PersonId,
  ) -> Result<usize, Error> {
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
  fn unresolve(
    conn: &PgConnection,
    report_id: Self::IdType,
    by_resolver_id: PersonId,
  ) -> Result<usize, Error> {
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
