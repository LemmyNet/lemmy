use crate::structs::PersonMentionView;
use diesel::{
  dsl::exists,
  pg::Pg,
  result::Error,
  sql_types,
  BoolExpressionMethods,
  BoxableExpression,
  ExpressionMethods,
  IntoSql,
  JoinOnDsl,
  NullableExpressionMethods,
  QueryDsl,
};
use diesel_async::RunQueryDsl;
use lemmy_db_schema::{
  aliases,
  newtypes::{PersonId, PersonMentionId},
  schema::{
    comment,
    comment_aggregates,
    comment_like,
    comment_saved,
    community,
    community_follower,
    community_moderator,
    community_person_ban,
    local_user,
    person,
    person_block,
    person_mention,
    post,
  },
  utils::{get_conn, limit_and_offset, DbConn, DbPool, ListFn, Queries, ReadFn},
  CommentSortType,
};

fn queries<'a>() -> Queries<
  impl ReadFn<'a, PersonMentionView, (PersonMentionId, Option<PersonId>)>,
  impl ListFn<'a, PersonMentionView, PersonMentionQuery>,
> {
  let is_creator_banned_from_community = exists(
    community_person_ban::table.filter(
      community::id
        .eq(community_person_ban::community_id)
        .and(community_person_ban::person_id.eq(comment::creator_id)),
    ),
  );

  let is_local_user_banned_from_community = |person_id| {
    exists(
      community_person_ban::table.filter(
        community::id
          .eq(community_person_ban::community_id)
          .and(community_person_ban::person_id.eq(person_id)),
      ),
    )
  };

  let is_saved = |person_id| {
    exists(
      comment_saved::table.filter(
        comment::id
          .eq(comment_saved::comment_id)
          .and(comment_saved::person_id.eq(person_id)),
      ),
    )
  };

  let is_community_followed = |person_id| {
    community_follower::table
      .filter(
        post::community_id
          .eq(community_follower::community_id)
          .and(community_follower::person_id.eq(person_id)),
      )
      .select(community_follower::pending.nullable())
      .single_value()
  };

  let is_creator_blocked = |person_id| {
    exists(
      person_block::table.filter(
        comment::creator_id
          .eq(person_block::target_id)
          .and(person_block::person_id.eq(person_id)),
      ),
    )
  };

  let score = |person_id| {
    comment_like::table
      .filter(
        comment::id
          .eq(comment_like::comment_id)
          .and(comment_like::person_id.eq(person_id)),
      )
      .select(comment_like::score.nullable())
      .single_value()
  };

  let creator_is_moderator = exists(
    community_moderator::table.filter(
      community::id
        .eq(community_moderator::community_id)
        .and(community_moderator::person_id.eq(comment::creator_id)),
    ),
  );

  let creator_is_admin = exists(
    local_user::table.filter(
      comment::creator_id
        .eq(local_user::person_id)
        .and(local_user::admin.eq(true)),
    ),
  );

  let all_joins = move |query: person_mention::BoxedQuery<'a, Pg>,
                        my_person_id: Option<PersonId>| {
    let is_local_user_banned_from_community_selection: Box<
      dyn BoxableExpression<_, Pg, SqlType = sql_types::Bool>,
    > = if let Some(person_id) = my_person_id {
      Box::new(is_local_user_banned_from_community(person_id))
    } else {
      Box::new(false.into_sql::<sql_types::Bool>())
    };
    let score_selection: Box<
      dyn BoxableExpression<_, Pg, SqlType = sql_types::Nullable<sql_types::SmallInt>>,
    > = if let Some(person_id) = my_person_id {
      Box::new(score(person_id))
    } else {
      Box::new(None::<i16>.into_sql::<sql_types::Nullable<sql_types::SmallInt>>())
    };

    let subscribed_type_selection: Box<
      dyn BoxableExpression<_, Pg, SqlType = sql_types::Nullable<sql_types::Bool>>,
    > = if let Some(person_id) = my_person_id {
      Box::new(is_community_followed(person_id))
    } else {
      Box::new(None::<bool>.into_sql::<sql_types::Nullable<sql_types::Bool>>())
    };

    let is_saved_selection: Box<dyn BoxableExpression<_, Pg, SqlType = sql_types::Bool>> =
      if let Some(person_id) = my_person_id {
        Box::new(is_saved(person_id))
      } else {
        Box::new(false.into_sql::<sql_types::Bool>())
      };

    let is_creator_blocked_selection: Box<dyn BoxableExpression<_, Pg, SqlType = sql_types::Bool>> =
      if let Some(person_id) = my_person_id {
        Box::new(is_creator_blocked(person_id))
      } else {
        Box::new(false.into_sql::<sql_types::Bool>())
      };

    query
      .inner_join(comment::table)
      .inner_join(person::table.on(comment::creator_id.eq(person::id)))
      .inner_join(post::table.on(comment::post_id.eq(post::id)))
      .inner_join(community::table.on(post::community_id.eq(community::id)))
      .inner_join(aliases::person1)
      .inner_join(comment_aggregates::table.on(comment::id.eq(comment_aggregates::comment_id)))
      .select((
        person_mention::all_columns,
        comment::all_columns,
        person::all_columns,
        post::all_columns,
        community::all_columns,
        aliases::person1.fields(person::all_columns),
        comment_aggregates::all_columns,
        is_creator_banned_from_community,
        is_local_user_banned_from_community_selection,
        creator_is_moderator,
        creator_is_admin,
        subscribed_type_selection,
        is_saved_selection,
        is_creator_blocked_selection,
        score_selection,
      ))
  };

  let read =
    move |mut conn: DbConn<'a>,
          (person_mention_id, my_person_id): (PersonMentionId, Option<PersonId>)| async move {
      all_joins(
        person_mention::table.find(person_mention_id).into_boxed(),
        my_person_id,
      )
      .first::<PersonMentionView>(&mut conn)
      .await
    };

  let list = move |mut conn: DbConn<'a>, options: PersonMentionQuery| async move {
    let mut query = all_joins(person_mention::table.into_boxed(), options.my_person_id);

    if let Some(recipient_id) = options.recipient_id {
      query = query.filter(person_mention::recipient_id.eq(recipient_id));
    }

    if options.unread_only {
      query = query.filter(person_mention::read.eq(false));
    }

    if !options.show_bot_accounts {
      query = query.filter(person::bot_account.eq(false));
    };

    query = match options.sort.unwrap_or(CommentSortType::Hot) {
      CommentSortType::Hot => query.then_order_by(comment_aggregates::hot_rank.desc()),
      CommentSortType::Controversial => {
        query.then_order_by(comment_aggregates::controversy_rank.desc())
      }
      CommentSortType::New => query.then_order_by(comment::published.desc()),
      CommentSortType::Old => query.then_order_by(comment::published.asc()),
      CommentSortType::Top => query.order_by(comment_aggregates::score.desc()),
    };

    let (limit, offset) = limit_and_offset(options.page, options.limit)?;

    query
      .limit(limit)
      .offset(offset)
      .load::<PersonMentionView>(&mut conn)
      .await
  };

  Queries::new(read, list)
}

impl PersonMentionView {
  pub async fn read(
    pool: &mut DbPool<'_>,
    person_mention_id: PersonMentionId,
    my_person_id: Option<PersonId>,
  ) -> Result<Self, Error> {
    queries()
      .read(pool, (person_mention_id, my_person_id))
      .await
  }

  /// Gets the number of unread mentions
  pub async fn get_unread_mentions(
    pool: &mut DbPool<'_>,
    my_person_id: PersonId,
  ) -> Result<i64, Error> {
    use diesel::dsl::count;
    let conn = &mut get_conn(pool).await?;

    person_mention::table
      .inner_join(comment::table)
      .filter(person_mention::recipient_id.eq(my_person_id))
      .filter(person_mention::read.eq(false))
      .filter(comment::deleted.eq(false))
      .filter(comment::removed.eq(false))
      .select(count(person_mention::id))
      .first::<i64>(conn)
      .await
  }
}

#[derive(Default)]
pub struct PersonMentionQuery {
  pub my_person_id: Option<PersonId>,
  pub recipient_id: Option<PersonId>,
  pub sort: Option<CommentSortType>,
  pub unread_only: bool,
  pub show_bot_accounts: bool,
  pub page: Option<i64>,
  pub limit: Option<i64>,
}

impl PersonMentionQuery {
  pub async fn list(self, pool: &mut DbPool<'_>) -> Result<Vec<PersonMentionView>, Error> {
    queries().list(pool, self).await
  }
}
