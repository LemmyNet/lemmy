use crate::{
  newtypes::{CommunityId, CommunityReportId, PersonId},
  schema::community_report::{
    community_id,
    dsl::{community_report, resolved, resolver_id, updated},
  },
  source::community_report::{CommunityReport, CommunityReportForm},
  traits::Reportable,
  utils::{get_conn, DbPool},
};
use chrono::Utc;
use diesel::{
  dsl::{insert_into, update},
  result::Error,
  ExpressionMethods,
  QueryDsl,
};
use diesel_async::RunQueryDsl;

#[async_trait]
impl Reportable for CommunityReport {
  type Form = CommunityReportForm;
  type IdType = CommunityReportId;
  type ObjectIdType = CommunityId;
  /// creates a community report and returns it
  ///
  /// * `conn` - the postgres connection
  /// * `community_report_form` - the filled CommunityReportForm to insert
  async fn report(
    pool: &mut DbPool<'_>,
    community_report_form: &CommunityReportForm,
  ) -> Result<Self, Error> {
    let conn = &mut get_conn(pool).await?;
    insert_into(community_report)
      .values(community_report_form)
      .get_result::<Self>(conn)
      .await
  }

  /// resolve a community report
  ///
  /// * `conn` - the postgres connection
  /// * `report_id` - the id of the report to resolve
  /// * `by_resolver_id` - the id of the user resolving the report
  async fn resolve(
    pool: &mut DbPool<'_>,
    report_id_: Self::IdType,
    by_resolver_id: PersonId,
  ) -> Result<usize, Error> {
    let conn = &mut get_conn(pool).await?;
    update(community_report.find(report_id_))
      .set((
        resolved.eq(true),
        resolver_id.eq(by_resolver_id),
        updated.eq(Utc::now()),
      ))
      .execute(conn)
      .await
  }

  async fn resolve_all_for_object(
    pool: &mut DbPool<'_>,
    community_id_: CommunityId,
    by_resolver_id: PersonId,
  ) -> Result<usize, Error> {
    let conn = &mut get_conn(pool).await?;
    update(community_report.filter(community_id.eq(community_id_)))
      .set((
        resolved.eq(true),
        resolver_id.eq(by_resolver_id),
        updated.eq(Utc::now()),
      ))
      .execute(conn)
      .await
  }

  /// unresolve a community report
  ///
  /// * `conn` - the postgres connection
  /// * `report_id` - the id of the report to unresolve
  /// * `by_resolver_id` - the id of the user unresolving the report
  async fn unresolve(
    pool: &mut DbPool<'_>,
    report_id_: Self::IdType,
    by_resolver_id: PersonId,
  ) -> Result<usize, Error> {
    let conn = &mut get_conn(pool).await?;
    update(community_report.find(report_id_))
      .set((
        resolved.eq(false),
        resolver_id.eq(by_resolver_id),
        updated.eq(Utc::now()),
      ))
      .execute(conn)
      .await
  }
}
