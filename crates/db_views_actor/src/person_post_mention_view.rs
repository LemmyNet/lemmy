use crate::structs::PersonPostMentionView;
use diesel::{
  dsl::exists,
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
  newtypes::{PersonId, PersonPostMentionId},
  schema::{
    community,
    community_actions,
    image_details,
    local_user,
    person,
    person_actions,
    person_post_mention,
    post,
    post_actions,
    post_aggregates,
  },
  source::community::CommunityFollower,
  utils::{actions, actions_alias, functions::coalesce, get_conn, DbPool},
};

impl PersonPostMentionView {
  pub async fn read(
    pool: &mut DbPool<'_>,
    person_post_mention_id: PersonPostMentionId,
    my_person_id: Option<PersonId>,
  ) -> Result<Self, Error> {
    let conn = &mut get_conn(pool).await?;

    let creator_is_admin = exists(
      local_user::table.filter(
        post::creator_id
          .eq(local_user::person_id)
          .and(local_user::admin.eq(true)),
      ),
    );

    person_post_mention::table
      .find(person_post_mention_id)
      .inner_join(post::table)
      .inner_join(person::table.on(post::creator_id.eq(person::id)))
      .inner_join(community::table.on(post::community_id.eq(community::id)))
      .inner_join(aliases::person1)
      .inner_join(post_aggregates::table.on(post::id.eq(post_aggregates::post_id)))
      .left_join(image_details::table.on(post::thumbnail_url.eq(image_details::link.nullable())))
      .left_join(actions(
        community_actions::table,
        my_person_id,
        post::community_id,
      ))
      .left_join(actions(post_actions::table, my_person_id, post::id))
      .left_join(actions(
        person_actions::table,
        my_person_id,
        post::creator_id,
      ))
      .left_join(actions_alias(
        creator_community_actions,
        post::creator_id,
        post::community_id,
      ))
      .select((
        person_post_mention::all_columns,
        post::all_columns,
        person::all_columns,
        community::all_columns,
        image_details::all_columns.nullable(),
        aliases::person1.fields(person::all_columns),
        post_aggregates::all_columns,
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
        post_actions::saved.nullable().is_not_null(),
        post_actions::read.nullable().is_not_null(),
        post_actions::hidden.nullable().is_not_null(),
        person_actions::blocked.nullable().is_not_null(),
        post_actions::like_score.nullable(),
        coalesce(
          post_aggregates::comments.nullable() - post_actions::read_comments_amount.nullable(),
          post_aggregates::comments,
        ),
      ))
      .first(conn)
      .await
  }
}
