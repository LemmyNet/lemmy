use crate::structs::FollowedCommunityPostView;
use diesel::{
  pg::Pg,
  result::Error,
  BoolExpressionMethods,
  ExpressionMethods,
  JoinOnDsl,
  NullableExpressionMethods,
  QueryDsl,
};
use diesel_async::RunQueryDsl;
use lemmy_db_schema::{
  newtypes::PersonId,
  schema::{community, community_follower, post, post_read},
  utils::{get_conn, limit_and_offset, DbConn, DbPool, ListFn, Queries, ReadFn},
  SortType,
};

#[derive(Default)]
pub struct FollowedCommunityPostQuery {
  pub my_person_id: Option<PersonId>,
  pub unread_only: bool,
  pub sort: Option<SortType>,
  pub page: Option<i64>,
  pub limit: Option<i64>,
}

fn queries<'a>() -> Queries<
  impl ReadFn<'a, FollowedCommunityPostView, PersonId>,
  impl ListFn<'a, FollowedCommunityPostView, FollowedCommunityPostQuery>,
> {
  let all_joins = |query: community_follower::BoxedQuery<'a, Pg>,
                   my_person_id: Option<PersonId>| {
    let person_id_join = my_person_id.unwrap_or(PersonId(-1));

    query
      .inner_join(
        community::table.on(
          community_follower::community_id
            .eq(community::id)
            .and(community_follower::person_id.eq(person_id_join)),
        ),
      )
      .inner_join(post::table.on(community::id.eq(post::community_id)))
      .left_join(
        post_read::table.on(
          post_read::post_id
            .eq(post::id)
            .and(post_read::person_id.eq(person_id_join)),
        ),
      )
      .filter(post::creator_id.ne(person_id_join))
  };

  let selection = (
    community::all_columns,
    community_follower::pending,
    community_follower::notifications_enabled,
    post::all_columns,
    post_read::id.nullable().is_not_null(),
  );

  let read = move |mut conn: DbConn<'a>, person_id: PersonId| async move {
    let query = all_joins(
      community_follower::table.find(person_id).into_boxed(),
      Some(person_id),
    )
    .select(selection);

    query.first::<FollowedCommunityPostView>(&mut conn).await
  };

  let list = move |mut conn: DbConn<'a>, options: FollowedCommunityPostQuery| async move {
    let mut query = all_joins(community_follower::table.into_boxed(), options.my_person_id);

    if options.unread_only {
      query = query.filter(post_read::id.nullable().is_null());
    }

    let (limit, offset) = limit_and_offset(options.page, options.limit)?;

    query
      .select(selection)
      .limit(limit)
      .offset(offset)
      .load::<FollowedCommunityPostView>(&mut conn)
      .await
  };

  Queries::new(read, list)
}

impl FollowedCommunityPostView {
  pub async fn read(pool: &mut DbPool<'_>, my_person_id: PersonId) -> Result<Self, Error> {
    queries().read(pool, my_person_id).await
  }

  // Gets the number of unread posts posted by a community that the user follows
  pub async fn get_unread_posts(
    pool: &mut DbPool<'_>,
    my_person_id: PersonId,
  ) -> Result<i64, Error> {
    use diesel::dsl::count;

    let conn = &mut get_conn(pool).await?;

    community_follower::table
      .inner_join(
        community::table.on(
          community_follower::community_id
            .eq(community::id)
            .and(community_follower::person_id.eq(my_person_id)),
        ),
      )
      .inner_join(post::table.on(community::id.eq(post::community_id)))
      .left_join(
        post_read::table.on(
          post_read::post_id
            .eq(post::id)
            .and(post_read::person_id.eq(my_person_id)),
        ),
      )
      .filter(post_read::id.is_not_null())
      .select(count(post::id))
      .first::<i64>(conn)
      .await
  }
}

impl FollowedCommunityPostQuery {
  pub async fn list(self, pool: &mut DbPool<'_>) -> Result<Vec<FollowedCommunityPostView>, Error> {
    queries().list(pool, self).await
  }
}
