use crate::{
  newtypes::{PersonId, PrivateMessageReportId},
  schema::private_message_report::dsl::{private_message_report, resolved, resolver_id, updated},
  source::private_message_report::{PrivateMessageReport, PrivateMessageReportForm},
  traits::Reportable,
  utils::{naive_now, DbConn},
};
use diesel::{
  dsl::{insert_into, update},
  result::Error,
  ExpressionMethods,
  QueryDsl,
};
use diesel_async::RunQueryDsl;

#[async_trait]
impl Reportable for PrivateMessageReport {
  type Form = PrivateMessageReportForm;
  type IdType = PrivateMessageReportId;

  async fn report(
    conn: &mut DbConn,
    pm_report_form: &PrivateMessageReportForm,
  ) -> Result<Self, Error> {
    insert_into(private_message_report)
      .values(pm_report_form)
      .get_result::<Self>(conn)
      .await
  }

  async fn resolve(
    conn: &mut DbConn,
    report_id: Self::IdType,
    by_resolver_id: PersonId,
  ) -> Result<usize, Error> {
    update(private_message_report.find(report_id))
      .set((
        resolved.eq(true),
        resolver_id.eq(by_resolver_id),
        updated.eq(naive_now()),
      ))
      .execute(conn)
      .await
  }

  async fn unresolve(
    conn: &mut DbConn,
    report_id: Self::IdType,
    by_resolver_id: PersonId,
  ) -> Result<usize, Error> {
    update(private_message_report.find(report_id))
      .set((
        resolved.eq(false),
        resolver_id.eq(by_resolver_id),
        updated.eq(naive_now()),
      ))
      .execute(conn)
      .await
  }
}
