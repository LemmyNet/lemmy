use crate::CommunityReportView;
use diesel::{
  BoolExpressionMethods,
  ExpressionMethods,
  JoinOnDsl,
  NullableExpressionMethods,
  QueryDsl,
  SelectableHelper,
};
use diesel_async::RunQueryDsl;
use lemmy_db_schema::{
  aliases,
  newtypes::{CommunityReportId, PersonId},
  utils::{get_conn, DbPool},
};
use lemmy_db_schema_file::schema::{community, community_actions, community_report, person};
use lemmy_utils::error::{LemmyErrorExt, LemmyErrorType, LemmyResult};

impl CommunityReportView {
  /// returns the CommunityReportView for the provided report_id
  ///
  /// * `report_id` - the report id to obtain
  pub async fn read(
    pool: &mut DbPool<'_>,
    report_id: CommunityReportId,
    my_person_id: PersonId,
  ) -> LemmyResult<Self> {
    let conn = &mut get_conn(pool).await?;

    let resolver_id = aliases::person2.field(person::id);

    let report_creator_join = person::table.on(community_report::creator_id.eq(person::id));
    let resolver_join =
      aliases::person2.on(community_report::resolver_id.eq(resolver_id.nullable()));

    let community_actions_join = community_actions::table.on(
      community_actions::community_id
        .eq(community_report::community_id)
        .and(community_actions::person_id.eq(my_person_id)),
    );

    community_report::table
      .find(report_id)
      .inner_join(community::table)
      .inner_join(report_creator_join)
      .left_join(resolver_join)
      .left_join(community_actions_join)
      .select(Self::as_select())
      .first(conn)
      .await
      .with_lemmy_type(LemmyErrorType::NotFound)
  }
}
