use crate::{
  newtypes::{CommentReportId, PersonId},
  schema::comment_report::dsl::{comment_report, resolved, resolver_id, updated},
  source::comment_report::{CommentReport, CommentReportForm},
  traits::Reportable,
  utils::{naive_now, GetConn},
};
use diesel::{
  dsl::{insert_into, update},
  result::Error,
  ExpressionMethods,
  QueryDsl,
};
use lemmy_db_schema::utils::RunQueryDsl;

#[async_trait]
impl Reportable for CommentReport {
  type Form = CommentReportForm;
  type IdType = CommentReportId;
  /// creates a comment report and returns it
  ///
  /// * `conn` - the postgres connection
  /// * `comment_report_form` - the filled CommentReportForm to insert
  async fn report(
    mut conn: impl GetConn,
    comment_report_form: &CommentReportForm,
  ) -> Result<Self, Error> {
    insert_into(comment_report)
      .values(comment_report_form)
      .get_result::<Self>(conn)
      .await
  }

  /// resolve a comment report
  ///
  /// * `conn` - the postgres connection
  /// * `report_id` - the id of the report to resolve
  /// * `by_resolver_id` - the id of the user resolving the report
  async fn resolve(
    mut conn: impl GetConn,
    report_id_: Self::IdType,
    by_resolver_id: PersonId,
  ) -> Result<usize, Error> {
    update(comment_report.find(report_id_))
      .set((
        resolved.eq(true),
        resolver_id.eq(by_resolver_id),
        updated.eq(naive_now()),
      ))
      .execute(conn)
      .await
  }

  /// unresolve a comment report
  ///
  /// * `conn` - the postgres connection
  /// * `report_id` - the id of the report to unresolve
  /// * `by_resolver_id` - the id of the user unresolving the report
  async fn unresolve(
    mut conn: impl GetConn,
    report_id_: Self::IdType,
    by_resolver_id: PersonId,
  ) -> Result<usize, Error> {
    update(comment_report.find(report_id_))
      .set((
        resolved.eq(false),
        resolver_id.eq(by_resolver_id),
        updated.eq(naive_now()),
      ))
      .execute(conn)
      .await
  }
}
