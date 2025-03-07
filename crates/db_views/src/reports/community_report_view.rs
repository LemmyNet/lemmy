use crate::structs::CommunityReportView;
use diesel::{
  result::Error,
  BoolExpressionMethods,
  ExpressionMethods,
  JoinOnDsl,
  NullableExpressionMethods,
  QueryDsl,
};
use diesel_async::RunQueryDsl;
use lemmy_db_schema::{
  aliases,
  impls::community::community_follower_select_subscribed_type,
  newtypes::{CommunityReportId, PersonId},
  schema::{community, community_actions, community_report, person},
  utils::{get_conn, DbPool},
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

    let community_actions_join = community_actions::table.on(
      community_actions::community_id
        .eq(community_report::community_id)
        .and(community_actions::person_id.eq(my_person_id)),
    );

    community_report::table
      .find(report_id)
      .inner_join(community::table)
      .inner_join(person::table.on(community_report::creator_id.eq(person::id)))
      .left_join(
        aliases::person2
          .on(community_report::resolver_id.eq(aliases::person2.field(person::id).nullable())),
      )
      .left_join(community_actions_join)
      .select((
        community_report::all_columns,
        community::all_columns,
        person::all_columns,
        community_follower_select_subscribed_type(),
        aliases::person2.fields(person::all_columns.nullable()),
      ))
      .first(conn)
      .await
  }
}
