use crate::{
  newtypes::{PersonId, PrivateMessageReportId},
  source::private_message_report::{PrivateMessageReport, PrivateMessageReportForm},
  traits::Reportable,
  utils::naive_now,
};
use diesel::{dsl::*, result::Error, *};

impl Reportable for PrivateMessageReport {
  type Form = PrivateMessageReportForm;
  type IdType = PrivateMessageReportId;
  /// creates a comment report and returns it
  ///
  /// * `conn` - the postgres connection
  /// * `comment_report_form` - the filled CommentReportForm to insert
  fn report(
    conn: &mut PgConnection,
    pm_report_form: &PrivateMessageReportForm,
  ) -> Result<Self, Error> {
    use crate::schema::private_message_report::dsl::*;
    insert_into(private_message_report)
      .values(pm_report_form)
      .get_result::<Self>(conn)
  }

  /// resolve a pm report
  ///
  /// * `conn` - the postgres connection
  /// * `report_id` - the id of the report to resolve
  /// * `by_resolver_id` - the id of the user resolving the report
  fn resolve(
    conn: &mut PgConnection,
    report_id: Self::IdType,
    by_resolver_id: PersonId,
  ) -> Result<usize, Error> {
    use crate::schema::private_message_report::dsl::*;
    update(private_message_report.find(report_id))
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
    conn: &mut PgConnection,
    report_id: Self::IdType,
    by_resolver_id: PersonId,
  ) -> Result<usize, Error> {
    use crate::schema::private_message_report::dsl::*;
    update(private_message_report.find(report_id))
      .set((
        resolved.eq(false),
        resolver_id.eq(by_resolver_id),
        updated.eq(naive_now()),
      ))
      .execute(conn)
  }
}
