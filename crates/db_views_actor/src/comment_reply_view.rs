use crate::structs::CommentReplyView;
use diesel::{
  pg::Pg,
  result::Error,
  BoolExpressionMethods,
  ExpressionMethods,
  JoinOnDsl,
  NullableExpressionMethods,
  QueryDsl,
  SelectableHelper,
};
use diesel_async::RunQueryDsl;
use lemmy_db_schema::{
  aggregates::structs::CommentAggregatesNotInComment,
  aliases,
  newtypes::{CommentReplyId, PersonId},
  schema::{
    comment,
    comment_aggregates,
    comment_like,
    comment_reply,
    comment_saved,
    community,
    community_follower,
    community_person_ban,
    person,
    person_block,
    post,
  },
  source::{
    comment::CommentWithoutId,
    comment_reply::CommentReply,
    community::{CommunityWithoutId, CommunityFollower},
    person::PersonWithoutId,
    post::PostWithoutId,
  },
  traits::JoinView,
  utils::{get_conn, limit_and_offset, DbConn, DbPool, ListFn, Queries, ReadFn},
  CommentSortType,
  SubscribedType,
};

type CommentReplyViewTuple = (
  CommentReply,
  CommentWithoutId,
  PersonWithoutId,
  PostWithoutId,
  CommunityWithoutId,
  PersonWithoutId,
  CommentAggregatesNotInComment,
  bool,
  SubscribedType,
  bool,
  bool,
  Option<i16>,
);

fn queries<'a>() -> Queries<
  impl ReadFn<'a, CommentReplyView, (CommentReplyId, Option<PersonId>)>,
  impl ListFn<'a, CommentReplyView, CommentReplyQuery>,
> {
  let all_joins = |query: comment_reply::BoxedQuery<'a, Pg>, my_person_id: Option<PersonId>| {
    // The left join below will return None in this case
    let person_id_join = my_person_id.unwrap_or(PersonId(-1));

    query
      .inner_join(comment::table)
      .inner_join(person::table.on(comment::creator_id.eq(person::id)))
      .inner_join(post::table.on(comment::post_id.eq(post::id)))
      .inner_join(community::table.on(post::community_id.eq(community::id)))
      .inner_join(aliases::person1)
      .inner_join(comment_aggregates::table.on(comment::id.eq(comment_aggregates::comment_id)))
      .left_join(
        community_person_ban::table.on(
          community::id
            .eq(community_person_ban::community_id)
            .and(community_person_ban::person_id.eq(comment::creator_id)),
        ),
      )
      .left_join(
        community_follower::table.on(
          post::community_id
            .eq(community_follower::community_id)
            .and(community_follower::person_id.eq(person_id_join)),
        ),
      )
      .left_join(
        comment_saved::table.on(
          comment::id
            .eq(comment_saved::comment_id)
            .and(comment_saved::person_id.eq(person_id_join)),
        ),
      )
      .left_join(
        person_block::table.on(
          comment::creator_id
            .eq(person_block::target_id)
            .and(person_block::person_id.eq(person_id_join)),
        ),
      )
      .left_join(
        comment_like::table.on(
          comment::id
            .eq(comment_like::comment_id)
            .and(comment_like::person_id.eq(person_id_join)),
        ),
      )
      .select((
        comment_reply::all_columns,
        CommentWithoutId::as_select(),
        PersonWithoutId::as_select(),
        PostWithoutId::as_select(),
        CommunityWithoutId::as_select(),
        aliases::person1.fields(PersonWithoutId::as_select()),
        CommentAggregatesNotInComment::as_select(),
        community_person_ban::id.nullable().is_not_null(),
        CommunityFollower::select_subscribed_type(),
        comment_saved::id.nullable().is_not_null(),
        person_block::id.nullable().is_not_null(),
        comment_like::score.nullable(),
      ))
  };

  let read =
    move |mut conn: DbConn<'a>,
          (comment_reply_id, my_person_id): (CommentReplyId, Option<PersonId>)| async move {
      all_joins(
        comment_reply::table.find(comment_reply_id).into_boxed(),
        my_person_id,
      )
      .first::<CommentReplyViewTuple>(&mut conn)
      .await
    };

  let list = move |mut conn: DbConn<'a>, options: CommentReplyQuery| async move {
    let mut query = all_joins(comment_reply::table.into_boxed(), options.my_person_id);

    if let Some(recipient_id) = options.recipient_id {
      query = query.filter(comment_reply::recipient_id.eq(recipient_id));
    }

    if options.unread_only.unwrap_or(false) {
      query = query.filter(comment_reply::read.eq(false));
    }

    if !options.show_bot_accounts.unwrap_or(true) {
      query = query.filter(person::bot_account.eq(false));
    };

    query = match options.sort.unwrap_or(CommentSortType::New) {
      CommentSortType::Hot => query.then_order_by(comment_aggregates::hot_rank.desc()),
      CommentSortType::Controversial => {
        query.then_order_by(comment_aggregates::controversy_rank.desc())
      }
      CommentSortType::New => query.then_order_by(comment_reply::published.desc()),
      CommentSortType::Old => query.then_order_by(comment_reply::published.asc()),
      CommentSortType::Top => query.order_by(comment_aggregates::score.desc()),
    };

    let (limit, offset) = limit_and_offset(options.page, options.limit)?;

    query
      .limit(limit)
      .offset(offset)
      .load::<CommentReplyViewTuple>(&mut conn)
      .await
  };

  Queries::new(read, list)
}

impl CommentReplyView {
  pub async fn read(
    pool: &mut DbPool<'_>,
    comment_reply_id: CommentReplyId,
    my_person_id: Option<PersonId>,
  ) -> Result<Self, Error> {
    queries().read(pool, (comment_reply_id, my_person_id)).await
  }

  /// Gets the number of unread replies
  pub async fn get_unread_replies(
    pool: &mut DbPool<'_>,
    my_person_id: PersonId,
  ) -> Result<i64, Error> {
    use diesel::dsl::count;

    let conn = &mut get_conn(pool).await?;

    comment_reply::table
      .inner_join(comment::table)
      .filter(comment_reply::recipient_id.eq(my_person_id))
      .filter(comment_reply::read.eq(false))
      .filter(comment::deleted.eq(false))
      .filter(comment::removed.eq(false))
      .select(count(comment_reply::id))
      .first::<i64>(conn)
      .await
  }
}

#[derive(Default)]
pub struct CommentReplyQuery {
  pub my_person_id: Option<PersonId>,
  pub recipient_id: Option<PersonId>,
  pub sort: Option<CommentSortType>,
  pub unread_only: Option<bool>,
  pub show_bot_accounts: Option<bool>,
  pub page: Option<i64>,
  pub limit: Option<i64>,
}

impl CommentReplyQuery {
  pub async fn list(self, pool: &mut DbPool<'_>) -> Result<Vec<CommentReplyView>, Error> {
    queries().list(pool, self).await
  }
}

impl JoinView for CommentReplyView {
  type JoinTuple = CommentReplyViewTuple;
  fn from_tuple(
    (
      comment_reply,
      comment,
      creator,
      post,
      community,
      recipient,
      counts,
      creator_banned_from_community,
      subscribed,
      saved,
      creator_blocked,
      my_vote,
    ): Self::JoinTuple,
  ) -> Self {
    Self {
      counts: counts.into_full(&comment),
      recipient: recipient.into_full(comment_reply.recipient_id),
      community: community.into_full(post.community_id),
      post: post.into_full(comment.post_id),
      creator: creator.into_full(comment.creator_id),
      comment: comment.into_full(comment_reply.comment_id),
      comment_reply,
      creator_banned_from_community,
      subscribed,
      saved,
      creator_blocked,
      my_vote,
    }
  }
}
