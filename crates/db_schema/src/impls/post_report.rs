use crate::{
  newtypes::{PersonId, PostReportId},
  schema::post_report::dsl::*,
  source::post_report::*,
  traits::Reportable,
  utils::{get_conn, naive_now, DbPool},
};
use diesel::{dsl::*, result::Error, ExpressionMethods, QueryDsl};
use diesel_async::RunQueryDsl;

#[async_trait]
impl Reportable for PostReport {
  type Form = PostReportForm;
  type IdType = PostReportId;

  async fn report(pool: &DbPool, post_report_form: &PostReportForm) -> Result<Self, Error> {
    let conn = &mut get_conn(&pool).await?;
    insert_into(post_report)
      .values(post_report_form)
      .get_result::<Self>(conn)
      .await
  }

  async fn resolve(
    pool: &DbPool,
    report_id: Self::IdType,
    by_resolver_id: PersonId,
  ) -> Result<usize, Error> {
    let conn = &mut get_conn(&pool).await?;
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
    pool: &DbPool,
    report_id: Self::IdType,
    by_resolver_id: PersonId,
  ) -> Result<usize, Error> {
    let conn = &mut get_conn(&pool).await?;
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
