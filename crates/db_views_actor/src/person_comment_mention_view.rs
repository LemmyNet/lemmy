use crate::structs::PersonCommentMentionView;
use diesel::{
  dsl::{exists, not},
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
  newtypes::{PersonCommentMentionId, PersonId},
  schema::{
    comment,
    comment_actions,
    comment_aggregates,
    community,
    community_actions,
    local_user,
    person,
    person_actions,
    person_comment_mention,
    post,
  },
  source::{community::CommunityFollower, local_user::LocalUser},
  utils::{actions, actions_alias, get_conn, DbPool},
};

impl PersonCommentMentionView {
  pub async fn read(
    pool: &mut DbPool<'_>,
    person_comment_mention_id: PersonCommentMentionId,
    my_person_id: Option<PersonId>,
  ) -> Result<Self, Error> {
    let conn = &mut get_conn(pool).await?;

    let creator_is_admin = exists(
      local_user::table.filter(
        comment::creator_id
          .eq(local_user::person_id)
          .and(local_user::admin.eq(true)),
      ),
    );

    person_comment_mention::table
      .find(person_comment_mention_id)
      .inner_join(comment::table)
      .inner_join(person::table.on(comment::creator_id.eq(person::id)))
      .inner_join(post::table.on(comment::post_id.eq(post::id)))
      .inner_join(community::table.on(post::community_id.eq(community::id)))
      .inner_join(aliases::person1)
      .inner_join(comment_aggregates::table.on(comment::id.eq(comment_aggregates::comment_id)))
      .left_join(actions(
        community_actions::table,
        my_person_id,
        post::community_id,
      ))
      .left_join(actions(comment_actions::table, my_person_id, comment::id))
      .left_join(actions(
        person_actions::table,
        my_person_id,
        comment::creator_id,
      ))
      .left_join(actions_alias(
        creator_community_actions,
        comment::creator_id,
        post::community_id,
      ))
      .select((
        person_comment_mention::all_columns,
        comment::all_columns,
        person::all_columns,
        post::all_columns,
        community::all_columns,
        aliases::person1.fields(person::all_columns),
        comment_aggregates::all_columns,
        creator_community_actions
          .field(community_actions::received_ban)
          .nullable()
          .is_not_null(),
        community_actions::received_ban.nullable().is_not_null(),
        creator_community_actions
          .field(community_actions::became_moderator)
          .nullable()
          .is_not_null(),
        creator_is_admin,
        CommunityFollower::select_subscribed_type(),
        comment_actions::saved.nullable().is_not_null(),
        person_actions::blocked.nullable().is_not_null(),
        comment_actions::like_score.nullable(),
      ))
      .first(conn)
      .await
  }

  /// Gets the number of unread mentions
  // TODO get rid of this
  pub async fn get_unread_count(
    pool: &mut DbPool<'_>,
    local_user: &LocalUser,
  ) -> Result<i64, Error> {
    use diesel::dsl::count;
    let conn = &mut get_conn(pool).await?;

    let mut query = person_comment_mention::table
      .inner_join(comment::table)
      .left_join(actions(
        person_actions::table,
        Some(local_user.person_id),
        comment::creator_id,
      ))
      .inner_join(person::table.on(comment::creator_id.eq(person::id)))
      .into_boxed();

    // These filters need to be kept in sync with the filters in queries().list()
    if !local_user.show_bot_accounts {
      query = query.filter(not(person::bot_account));
    }

    query
      // Don't count replies from blocked users
      .filter(person_actions::blocked.is_null())
      .filter(person_comment_mention::recipient_id.eq(local_user.person_id))
      .filter(person_comment_mention::read.eq(false))
      .filter(comment::deleted.eq(false))
      .filter(comment::removed.eq(false))
      .select(count(person_comment_mention::id))
      .first::<i64>(conn)
      .await
  }
}
