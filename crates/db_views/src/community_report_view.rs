use crate::structs::CommunityReportView;
use diesel::{result::Error, ExpressionMethods, JoinOnDsl, NullableExpressionMethods, QueryDsl};
use diesel_async::RunQueryDsl;
use lemmy_db_schema::{
  aliases::{self},
  newtypes::{CommunityReportId, PersonId},
  schema::{community, community_actions, community_aggregates, community_report, person},
  source::community::CommunityFollower,
  utils::{actions, get_conn, DbPool},
};

impl CommunityReportView {
  /// returns the CommunityReportView for the provided report_id
  ///
  /// * `report_id` - the report id to obtain
  pub async fn read(
    pool: &mut DbPool<'_>,
    report_id: CommunityReportId,
    my_person_id: PersonId,
  ) -> Result<Self, Error> {
    let conn = &mut get_conn(pool).await?;

    community_report::table
      .find(report_id)
      .inner_join(community::table.inner_join(community_aggregates::table))
      .left_join(actions(
        community_actions::table,
        Some(my_person_id),
        community_report::community_id,
      ))
      .inner_join(
        aliases::person1.on(community_report::creator_id.eq(aliases::person1.field(person::id))),
      )
      .left_join(
        aliases::person2
          .on(community_report::resolver_id.eq(aliases::person2.field(person::id).nullable())),
      )
      .select((
        community_report::all_columns,
        community::all_columns,
        aliases::person1.fields(person::all_columns),
        community_aggregates::all_columns,
        CommunityFollower::select_subscribed_type(),
        aliases::person2.fields(person::all_columns.nullable()),
      ))
      .first(conn)
      .await
  }
}
