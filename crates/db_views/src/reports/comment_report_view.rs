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
  impls::community::community_follower_select_subscribed_type,
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
  utils::{functions::coalesce, get_conn, DbPool},
};

impl CommentReportView {
  #[diesel::dsl::auto_type(no_type_alias)]
  fn joins(my_person_id: PersonId) -> _ {
    let recipient_id = aliases::person1.field(person::id);
    let resolver_id = aliases::person2.field(person::id);

    let post_join = post::table.on(comment::post_id.eq(post::id));

    let community_join = community::table.on(post::community_id.eq(community::id));

    let report_creator_join = person::table.on(comment_report::creator_id.eq(person::id));

    let local_user_join = local_user::table.on(
      comment::creator_id
        .eq(local_user::person_id)
        .and(local_user::admin.eq(true)),
    );

    let comment_creator_join = aliases::person1.on(comment::creator_id.eq(recipient_id));

    let comment_aggregates_join =
      comment_aggregates::table.on(comment_report::comment_id.eq(comment_aggregates::comment_id));

    let comment_actions_join = comment_actions::table.on(
      comment_actions::comment_id
        .eq(comment_report::comment_id)
        .and(comment_actions::person_id.eq(my_person_id)),
    );

    let resolver_join = aliases::person2.on(comment_report::resolver_id.eq(resolver_id.nullable()));

    let creator_community_actions_join = creator_community_actions.on(
      creator_community_actions
        .field(community_actions::community_id)
        .eq(post::community_id)
        .and(
          creator_community_actions
            .field(community_actions::person_id)
            .eq(comment::creator_id),
        ),
    );

    let person_actions_join = person_actions::table.on(
      person_actions::target_id
        .eq(comment::creator_id)
        .and(person_actions::person_id.eq(my_person_id)),
    );

    let community_actions_join = community_actions::table.on(
      community_actions::community_id
        .eq(post::community_id)
        .and(community_actions::person_id.eq(my_person_id)),
    );

    comment_report::table
      .inner_join(comment::table)
      .inner_join(post_join)
      .inner_join(community_join)
      .inner_join(report_creator_join)
      .inner_join(comment_creator_join)
      .inner_join(comment_aggregates_join)
      .left_join(comment_actions_join)
      .left_join(resolver_join)
      .left_join(creator_community_actions_join)
      .left_join(local_user_join)
      .left_join(person_actions_join)
      .left_join(community_actions_join)
  }

  /// returns the CommentReportView for the provided report_id
  ///
  /// * `report_id` - the report id to obtain
  pub async fn read(
    pool: &mut DbPool<'_>,
    report_id: CommentReportId,
    my_person_id: PersonId,
  ) -> Result<Self, Error> {
    let conn = &mut get_conn(pool).await?;
    Self::joins(my_person_id)
      .filter(comment_report::id.eq(report_id))
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
        community_follower_select_subscribed_type(),
        comment_actions::saved.nullable(),
        comment_actions::like_score.nullable(),
        aliases::person2.fields(person::all_columns).nullable(),
      ))
      .first(conn)
      .await
  }
}
