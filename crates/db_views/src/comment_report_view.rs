use crate::structs::CommentReportView;
use diesel::{
  dsl::now,
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
  newtypes::{CommentReportId, PersonId},
  schema::{
    comment,
    comment_actions,
    comment_aggregates,
    comment_report,
    community,
    community_actions,
    local_user,
    person,
    person_actions,
    post,
  },
  source::community::CommunityFollower,
  utils::{actions, actions_alias, functions::coalesce, get_conn, DbPool},
};

impl CommentReportView {
  /// returns the CommentReportView for the provided report_id
  ///
  /// * `report_id` - the report id to obtain
  pub async fn read(
    pool: &mut DbPool<'_>,
    report_id: CommentReportId,
    my_person_id: PersonId,
  ) -> Result<Self, Error> {
    let conn = &mut get_conn(pool).await?;
    comment_report::table
      .find(report_id)
      .inner_join(comment::table)
      .inner_join(post::table.on(comment::post_id.eq(post::id)))
      .inner_join(community::table.on(post::community_id.eq(community::id)))
      .inner_join(person::table.on(comment_report::creator_id.eq(person::id)))
      .inner_join(aliases::person1.on(comment::creator_id.eq(aliases::person1.field(person::id))))
      .inner_join(
        comment_aggregates::table.on(comment_report::comment_id.eq(comment_aggregates::comment_id)),
      )
      .left_join(actions(
        comment_actions::table,
        Some(my_person_id),
        comment_report::comment_id,
      ))
      .left_join(
        aliases::person2
          .on(comment_report::resolver_id.eq(aliases::person2.field(person::id).nullable())),
      )
      .left_join(actions_alias(
        creator_community_actions,
        comment::creator_id,
        post::community_id,
      ))
      .left_join(
        local_user::table.on(
          comment::creator_id
            .eq(local_user::person_id)
            .and(local_user::admin.eq(true)),
        ),
      )
      .left_join(actions(
        person_actions::table,
        Some(my_person_id),
        comment::creator_id,
      ))
      .left_join(actions(
        community_actions::table,
        Some(my_person_id),
        post::community_id,
      ))
      .select((
        comment_report::all_columns,
        comment::all_columns,
        post::all_columns,
        community::all_columns,
        person::all_columns,
        aliases::person1.fields(person::all_columns),
        comment_aggregates::all_columns,
        coalesce(
          creator_community_actions
            .field(community_actions::received_ban)
            .nullable()
            .is_not_null()
            .or(
              creator_community_actions
                .field(community_actions::ban_expires)
                .nullable()
                .gt(now),
            ),
          false,
        ),
        creator_community_actions
          .field(community_actions::became_moderator)
          .nullable()
          .is_not_null(),
        local_user::admin.nullable().is_not_null(),
        person_actions::blocked.nullable().is_not_null(),
        CommunityFollower::select_subscribed_type(),
        comment_actions::saved.nullable().is_not_null(),
        comment_actions::like_score.nullable(),
        aliases::person2.fields(person::all_columns).nullable(),
      ))
      .first(conn)
      .await
  }
}
