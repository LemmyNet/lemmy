use crate::{
  newtypes::{CommunityId, CommunityReportId, PersonId},
  schema::community_report,
  source::community_report::{CommunityReport, CommunityReportForm},
  traits::Reportable,
  utils::{get_conn, DbPool},
};
use chrono::Utc;
use diesel::{
  dsl::{insert_into, update},
  result::Error,
  BoolExpressionMethods,
  ExpressionMethods,
  QueryDsl,
};
use diesel_async::RunQueryDsl;
use lemmy_utils::error::LemmyResult;

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
    insert_into(community_report::table)
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
    update(community_report::table.find(report_id_))
      .set((
        community_report::resolved.eq(true),
        community_report::resolver_id.eq(by_resolver_id),
        community_report::updated.eq(Utc::now()),
      ))
      .execute(conn)
      .await
  }

  async fn resolve_apub(
    pool: &mut DbPool<'_>,
    object_id: Self::ObjectIdType,
    report_creator_id: PersonId,
    resolver_id: PersonId,
  ) -> LemmyResult<usize> {
    let conn = &mut get_conn(pool).await?;
    Ok(
      update(
        community_report::table.filter(
          community_report::community_id
            .eq(object_id)
            .and(community_report::creator_id.eq(report_creator_id)),
        ),
      )
      .set((
        community_report::resolved.eq(true),
        community_report::resolver_id.eq(resolver_id),
        community_report::updated.eq(Utc::now()),
      ))
      .execute(conn)
      .await?,
    )
  }

  async fn resolve_all_for_object(
    pool: &mut DbPool<'_>,
    community_id_: CommunityId,
    by_resolver_id: PersonId,
  ) -> Result<usize, Error> {
    let conn = &mut get_conn(pool).await?;
    update(community_report::table.filter(community_report::community_id.eq(community_id_)))
      .set((
        community_report::resolved.eq(true),
        community_report::resolver_id.eq(by_resolver_id),
        community_report::updated.eq(Utc::now()),
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
    update(community_report::table.find(report_id_))
      .set((
        community_report::resolved.eq(false),
        community_report::resolver_id.eq(by_resolver_id),
        community_report::updated.eq(Utc::now()),
      ))
      .execute(conn)
      .await
  }
}
