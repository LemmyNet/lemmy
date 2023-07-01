use crate::{
  newtypes::{PersonId, PostReportId},
  schema::post_report::dsl::{post_report, resolved, resolver_id, updated},
  source::post_report::{PostReport, PostReportForm},
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
impl Reportable for PostReport {
  type Form = PostReportForm;
  type IdType = PostReportId;

  async fn report(mut conn: impl DbConn, post_report_form: &PostReportForm) -> Result<Self, Error> {
    insert_into(post_report)
      .values(post_report_form)
      .get_result::<Self>(&mut *conn)
      .await
  }

  async fn resolve(
    mut conn: impl DbConn,
    report_id: Self::IdType,
    by_resolver_id: PersonId,
  ) -> Result<usize, Error> {
    update(post_report.find(report_id))
      .set((
        resolved.eq(true),
        resolver_id.eq(by_resolver_id),
        updated.eq(naive_now()),
      ))
      .execute(&mut *conn)
      .await
  }

  async fn unresolve(
    mut conn: impl DbConn,
    report_id: Self::IdType,
    by_resolver_id: PersonId,
  ) -> Result<usize, Error> {
    update(post_report.find(report_id))
      .set((
        resolved.eq(false),
        resolver_id.eq(by_resolver_id),
        updated.eq(naive_now()),
      ))
      .execute(&mut *conn)
      .await
  }
}
