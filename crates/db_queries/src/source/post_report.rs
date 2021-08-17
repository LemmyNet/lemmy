use crate::Reportable;
use diesel::{dsl::*, result::Error, *};
use lemmy_db_schema::{naive_now, source::post_report::*, PersonId};

impl Reportable for PostReport {
  type Form = PostReportForm;
  /// creates a post report and returns it
  ///
  /// * `conn` - the postgres connection
  /// * `post_report_form` - the filled CommentReportForm to insert
  fn report(conn: &PgConnection, post_report_form: &PostReportForm) -> Result<Self, Error> {
    use lemmy_db_schema::schema::post_report::dsl::*;
    insert_into(post_report)
      .values(post_report_form)
      .get_result::<Self>(conn)
  }

  /// resolve a post report
  ///
  /// * `conn` - the postgres connection
  /// * `report_id` - the id of the report to resolve
  /// * `by_resolver_id` - the id of the user resolving the report
  fn resolve(
    conn: &PgConnection,
    report_id: i32,
    by_resolver_id: PersonId,
  ) -> Result<usize, Error> {
    use lemmy_db_schema::schema::post_report::dsl::*;
    update(post_report.find(report_id))
      .set((
        resolved.eq(true),
        resolver_id.eq(by_resolver_id),
        updated.eq(naive_now()),
      ))
      .execute(conn)
  }

  /// resolve a post report
  ///
  /// * `conn` - the postgres connection
  /// * `report_id` - the id of the report to unresolve
  /// * `by_resolver_id` - the id of the user unresolving the report
  fn unresolve(
    conn: &PgConnection,
    report_id: i32,
    by_resolver_id: PersonId,
  ) -> Result<usize, Error> {
    use lemmy_db_schema::schema::post_report::dsl::*;
    update(post_report.find(report_id))
      .set((
        resolved.eq(false),
        resolver_id.eq(by_resolver_id),
        updated.eq(naive_now()),
      ))
      .execute(conn)
  }
}
