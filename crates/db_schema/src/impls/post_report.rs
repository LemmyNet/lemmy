use crate::{
  newtypes::{PersonId, PostReportId},
  schema::post_report::dsl::{post_report, resolved, resolver_id, updated},
  source::post_report::{PostReport, PostReportForm},
  traits::Reportable,
  utils::{naive_now, DbPool, GetConn},
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

  async fn report(
    mut pool: &mut impl GetConn,
    post_report_form: &PostReportForm,
  ) -> Result<Self, Error> {
    let conn = &mut *pool.get_conn().await?;
    insert_into(post_report)
      .values(post_report_form)
      .get_result::<Self>(conn)
      .await
  }

  async fn resolve(
    mut pool: &mut impl GetConn,
    report_id: Self::IdType,
    by_resolver_id: PersonId,
  ) -> Result<usize, Error> {
    let conn = &mut *pool.get_conn().await?;
    update(post_report.find(report_id))
      .set((
        resolved.eq(true),
        resolver_id.eq(by_resolver_id),
        updated.eq(naive_now()),
      ))
      .execute(conn)
      .await
  }

  async fn unresolve(
    mut pool: &mut impl GetConn,
    report_id: Self::IdType,
    by_resolver_id: PersonId,
  ) -> Result<usize, Error> {
    let conn = &mut *pool.get_conn().await?;
    update(post_report.find(report_id))
      .set((
        resolved.eq(false),
        resolver_id.eq(by_resolver_id),
        updated.eq(naive_now()),
      ))
      .execute(conn)
      .await
  }
}
