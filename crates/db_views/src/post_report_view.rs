use crate::structs::PostReportView;
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
  aliases::{self, creator_community_actions},
  newtypes::{PersonId, PostReportId},
  schema::{
    community,
    community_actions,
    local_user,
    person,
    person_actions,
    post,
    post_actions,
    post_aggregates,
    post_report,
  },
  source::community::CommunityFollower,
  utils::{actions, actions_alias, functions::coalesce, get_conn, DbPool},
};

impl PostReportView {
  /// returns the PostReportView for the provided report_id
  ///
  /// * `report_id` - the report id to obtain
  pub async fn read(
    pool: &mut DbPool<'_>,
    report_id: PostReportId,
    my_person_id: PersonId,
  ) -> Result<Self, Error> {
    let conn = &mut get_conn(pool).await?;

    post_report::table
      .find(report_id)
      .inner_join(post::table)
      .inner_join(community::table.on(post::community_id.eq(community::id)))
      .inner_join(person::table.on(post_report::creator_id.eq(person::id)))
      .inner_join(aliases::person1.on(post::creator_id.eq(aliases::person1.field(person::id))))
      .left_join(actions_alias(
        creator_community_actions,
        post::creator_id,
        post::community_id,
      ))
      .left_join(actions(
        community_actions::table,
        Some(my_person_id),
        post::community_id,
      ))
      .left_join(
        local_user::table.on(
          post::creator_id
            .eq(local_user::person_id)
            .and(local_user::admin.eq(true)),
        ),
      )
      .left_join(actions(post_actions::table, Some(my_person_id), post::id))
      .left_join(actions(
        person_actions::table,
        Some(my_person_id),
        post::creator_id,
      ))
      .inner_join(post_aggregates::table.on(post_report::post_id.eq(post_aggregates::post_id)))
      .left_join(
        aliases::person2
          .on(post_report::resolver_id.eq(aliases::person2.field(person::id).nullable())),
      )
      .select((
        post_report::all_columns,
        post::all_columns,
        community::all_columns,
        person::all_columns,
        aliases::person1.fields(person::all_columns),
        creator_community_actions
          .field(community_actions::received_ban)
          .nullable()
          .is_not_null(),
        creator_community_actions
          .field(community_actions::became_moderator)
          .nullable()
          .is_not_null(),
        local_user::admin.nullable().is_not_null(),
        CommunityFollower::select_subscribed_type(),
        post_actions::saved.nullable().is_not_null(),
        post_actions::read.nullable().is_not_null(),
        post_actions::hidden.nullable().is_not_null(),
        person_actions::blocked.nullable().is_not_null(),
        post_actions::like_score.nullable(),
        coalesce(
          post_aggregates::comments.nullable() - post_actions::read_comments_amount.nullable(),
          post_aggregates::comments,
        ),
        post_aggregates::all_columns,
        aliases::person2.fields(person::all_columns.nullable()),
      ))
      .first(conn)
      .await
  }
}
