use crate::structs::CommentReplyView;
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
  aggregates::structs::CommentAggregates,
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
    comment::{Comment, CommentSaved},
    comment_reply::CommentReply,
    community::{Community, CommunityFollower, CommunityPersonBan, CommunitySafe},
    person::{Person, PersonSafe},
    person_block::PersonBlock,
    post::Post,
  },
  traits::{ToSafe, ViewToVec},
  utils::{functions::hot_rank, get_conn, limit_and_offset, DbPool},
  CommentSortType,
};
use typed_builder::TypedBuilder;

type CommentReplyViewTuple = (
  CommentReply,
  Comment,
  PersonSafe,
  Post,
  CommunitySafe,
  PersonSafe,
  CommentAggregates,
  Option<CommunityPersonBan>,
  Option<CommunityFollower>,
  Option<CommentSaved>,
  Option<PersonBlock>,
  Option<i16>,
);

impl CommentReplyView {
  pub async fn read(
    pool: &DbPool,
    comment_reply_id: CommentReplyId,
    my_person_id: Option<PersonId>,
  ) -> Result<Self, Error> {
    let conn = &mut get_conn(pool).await?;
    let person_alias_1 = diesel::alias!(person as person1);

    // The left join below will return None in this case
    let person_id_join = my_person_id.unwrap_or(PersonId(-1));

    let (
      comment_reply,
      comment,
      creator,
      post,
      community,
      recipient,
      counts,
      creator_banned_from_community,
      follower,
      saved,
      creator_blocked,
      my_vote,
    ) = comment_reply::table
      .find(comment_reply_id)
      .inner_join(comment::table)
      .inner_join(person::table.on(comment::creator_id.eq(person::id)))
      .inner_join(post::table.on(comment::post_id.eq(post::id)))
      .inner_join(community::table.on(post::community_id.eq(community::id)))
      .inner_join(person_alias_1)
      .inner_join(comment_aggregates::table.on(comment::id.eq(comment_aggregates::comment_id)))
      .left_join(
        community_person_ban::table.on(
          community::id
            .eq(community_person_ban::community_id)
            .and(community_person_ban::person_id.eq(comment::creator_id))
            .and(
              community_person_ban::expires
                .is_null()
                .or(community_person_ban::expires.gt(now)),
            ),
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
        comment::all_columns,
        Person::safe_columns_tuple(),
        post::all_columns,
        Community::safe_columns_tuple(),
        person_alias_1.fields(Person::safe_columns_tuple()),
        comment_aggregates::all_columns,
        community_person_ban::all_columns.nullable(),
        community_follower::all_columns.nullable(),
        comment_saved::all_columns.nullable(),
        person_block::all_columns.nullable(),
        comment_like::score.nullable(),
      ))
      .first::<CommentReplyViewTuple>(conn)
      .await?;

    Ok(CommentReplyView {
      comment_reply,
      comment,
      creator,
      post,
      community,
      recipient,
      counts,
      creator_banned_from_community: creator_banned_from_community.is_some(),
      subscribed: CommunityFollower::to_subscribed_type(&follower),
      saved: saved.is_some(),
      creator_blocked: creator_blocked.is_some(),
      my_vote,
    })
  }

  /// Gets the number of unread replies
  pub async fn get_unread_replies(pool: &DbPool, my_person_id: PersonId) -> Result<i64, Error> {
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

#[derive(TypedBuilder)]
#[builder(field_defaults(default))]
pub struct CommentReplyQuery<'a> {
  #[builder(!default)]
  pool: &'a DbPool,
  my_person_id: Option<PersonId>,
  recipient_id: Option<PersonId>,
  sort: Option<CommentSortType>,
  unread_only: Option<bool>,
  show_bot_accounts: Option<bool>,
  page: Option<i64>,
  limit: Option<i64>,
}

impl<'a> CommentReplyQuery<'a> {
  pub async fn list(self) -> Result<Vec<CommentReplyView>, Error> {
    let conn = &mut get_conn(self.pool).await?;

    let person_alias_1 = diesel::alias!(person as person1);

    // The left join below will return None in this case
    let person_id_join = self.my_person_id.unwrap_or(PersonId(-1));

    let mut query = comment_reply::table
      .inner_join(comment::table)
      .inner_join(person::table.on(comment::creator_id.eq(person::id)))
      .inner_join(post::table.on(comment::post_id.eq(post::id)))
      .inner_join(community::table.on(post::community_id.eq(community::id)))
      .inner_join(person_alias_1)
      .inner_join(comment_aggregates::table.on(comment::id.eq(comment_aggregates::comment_id)))
      .left_join(
        community_person_ban::table.on(
          community::id
            .eq(community_person_ban::community_id)
            .and(community_person_ban::person_id.eq(comment::creator_id))
            .and(
              community_person_ban::expires
                .is_null()
                .or(community_person_ban::expires.gt(now)),
            ),
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
        comment::all_columns,
        Person::safe_columns_tuple(),
        post::all_columns,
        Community::safe_columns_tuple(),
        person_alias_1.fields(Person::safe_columns_tuple()),
        comment_aggregates::all_columns,
        community_person_ban::all_columns.nullable(),
        community_follower::all_columns.nullable(),
        comment_saved::all_columns.nullable(),
        person_block::all_columns.nullable(),
        comment_like::score.nullable(),
      ))
      .into_boxed();

    if let Some(recipient_id) = self.recipient_id {
      query = query.filter(comment_reply::recipient_id.eq(recipient_id));
    }

    if self.unread_only.unwrap_or(false) {
      query = query.filter(comment_reply::read.eq(false));
    }

    if !self.show_bot_accounts.unwrap_or(true) {
      query = query.filter(person::bot_account.eq(false));
    };

    query = match self.sort.unwrap_or(CommentSortType::Hot) {
      CommentSortType::Hot => query
        .then_order_by(hot_rank(comment_aggregates::score, comment_aggregates::published).desc())
        .then_order_by(comment_aggregates::published.desc()),
      CommentSortType::New => query.then_order_by(comment::published.desc()),
      CommentSortType::Old => query.then_order_by(comment::published.asc()),
      CommentSortType::Top => query.order_by(comment_aggregates::score.desc()),
    };

    let (limit, offset) = limit_and_offset(self.page, self.limit)?;

    let res = query
      .limit(limit)
      .offset(offset)
      .load::<CommentReplyViewTuple>(conn)
      .await?;

    Ok(CommentReplyView::from_tuple_to_vec(res))
  }
}

impl ViewToVec for CommentReplyView {
  type DbTuple = CommentReplyViewTuple;
  fn from_tuple_to_vec(items: Vec<Self::DbTuple>) -> Vec<Self> {
    items
      .into_iter()
      .map(|a| Self {
        comment_reply: a.0,
        comment: a.1,
        creator: a.2,
        post: a.3,
        community: a.4,
        recipient: a.5,
        counts: a.6,
        creator_banned_from_community: a.7.is_some(),
        subscribed: CommunityFollower::to_subscribed_type(&a.8),
        saved: a.9.is_some(),
        creator_blocked: a.10.is_some(),
        my_vote: a.11,
      })
      .collect::<Vec<Self>>()
  }
}
