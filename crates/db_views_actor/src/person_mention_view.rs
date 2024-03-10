use crate::structs::PersonMentionView;
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
  aliases::{self, creator_community_actions},
  newtypes::{PersonId, PersonMentionId},
  schema::{
    comment,
    comment_actions,
    comment_aggregates,
    community,
    community_actions,
    local_user,
    person,
    person_actions,
    person_mention,
    post,
  },
  source::community::CommunityFollower,
  utils::{
    actions,
    functions::coalesce,
    get_conn,
    limit_and_offset,
    DbConn,
    DbPool,
    ListFn,
    Queries,
    ReadFn,
  },
  CommentSortType,
};

fn queries<'a>() -> Queries<
  impl ReadFn<'a, PersonMentionView, (PersonMentionId, Option<PersonId>)>,
  impl ListFn<'a, PersonMentionView, PersonMentionQuery>,
> {
  let all_joins = move |query: person_mention::BoxedQuery<'a, Pg>,
                        my_person_id: Option<PersonId>| {
    query
      .inner_join(
        comment::table
          .inner_join(person::table.left_join(local_user::table))
          .inner_join(post::table.inner_join(community::table))
          .inner_join(comment_aggregates::table),
      )
      .inner_join(aliases::person1)
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
      .left_join(
        creator_community_actions.on(
          creator_community_actions
            .field(community_actions::person_id)
            .eq(comment::creator_id)
            .and(
              creator_community_actions
                .field(community_actions::community_id)
                .eq(post::community_id),
            ),
        ),
      )
      .select((
        person_mention::all_columns,
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
        creator_community_actions
          .field(community_actions::became_moderator)
          .nullable()
          .is_not_null(),
        coalesce(local_user::admin.nullable(), false),
        CommunityFollower::select_subscribed_type(),
        comment_actions::saved.nullable().is_not_null(),
        person_actions::blocked.nullable().is_not_null(),
        comment_actions::like_score.nullable(),
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
