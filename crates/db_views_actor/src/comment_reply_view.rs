use crate::structs::CommentReplyView;
use diesel::{dsl::*, result::Error, *};
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
    person_alias_1,
    person_block,
    post,
  },
  source::{
    comment::{Comment, CommentSaved},
    comment_reply::CommentReply,
    community::{Community, CommunityFollower, CommunityPersonBan, CommunitySafe},
    person::{Person, PersonAlias1, PersonSafe, PersonSafeAlias1},
    person_block::PersonBlock,
    post::Post,
  },
  traits::{MaybeOptional, ToSafe, ViewToVec},
  utils::{functions::hot_rank, limit_and_offset},
  SortType,
};

type CommentReplyViewTuple = (
  CommentReply,
  Comment,
  PersonSafe,
  Post,
  CommunitySafe,
  PersonSafeAlias1,
  CommentAggregates,
  Option<CommunityPersonBan>,
  Option<CommunityFollower>,
  Option<CommentSaved>,
  Option<PersonBlock>,
  Option<i16>,
);

impl CommentReplyView {
  pub fn read(
    conn: &PgConnection,
    comment_reply_id: CommentReplyId,
    my_person_id: Option<PersonId>,
  ) -> Result<Self, Error> {
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
      .inner_join(person_alias_1::table)
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
        PersonAlias1::safe_columns_tuple(),
        comment_aggregates::all_columns,
        community_person_ban::all_columns.nullable(),
        community_follower::all_columns.nullable(),
        comment_saved::all_columns.nullable(),
        person_block::all_columns.nullable(),
        comment_like::score.nullable(),
      ))
      .first::<CommentReplyViewTuple>(conn)?;

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
  pub fn get_unread_replies(conn: &PgConnection, my_person_id: PersonId) -> Result<i64, Error> {
    use diesel::dsl::*;

    comment_reply::table
      .filter(comment_reply::recipient_id.eq(my_person_id))
      .filter(comment_reply::read.eq(false))
      .select(count(comment_reply::id))
      .first::<i64>(conn)
  }
}

pub struct CommentReplyQueryBuilder<'a> {
  conn: &'a PgConnection,
  my_person_id: Option<PersonId>,
  recipient_id: Option<PersonId>,
  sort: Option<SortType>,
  unread_only: Option<bool>,
  show_bot_accounts: Option<bool>,
  page: Option<i64>,
  limit: Option<i64>,
}

impl<'a> CommentReplyQueryBuilder<'a> {
  pub fn create(conn: &'a PgConnection) -> Self {
    CommentReplyQueryBuilder {
      conn,
      my_person_id: None,
      recipient_id: None,
      sort: None,
      unread_only: None,
      show_bot_accounts: None,
      page: None,
      limit: None,
    }
  }

  pub fn sort<T: MaybeOptional<SortType>>(mut self, sort: T) -> Self {
    self.sort = sort.get_optional();
    self
  }

  pub fn unread_only<T: MaybeOptional<bool>>(mut self, unread_only: T) -> Self {
    self.unread_only = unread_only.get_optional();
    self
  }

  pub fn show_bot_accounts<T: MaybeOptional<bool>>(mut self, show_bot_accounts: T) -> Self {
    self.show_bot_accounts = show_bot_accounts.get_optional();
    self
  }

  pub fn recipient_id<T: MaybeOptional<PersonId>>(mut self, recipient_id: T) -> Self {
    self.recipient_id = recipient_id.get_optional();
    self
  }

  pub fn my_person_id<T: MaybeOptional<PersonId>>(mut self, my_person_id: T) -> Self {
    self.my_person_id = my_person_id.get_optional();
    self
  }

  pub fn page<T: MaybeOptional<i64>>(mut self, page: T) -> Self {
    self.page = page.get_optional();
    self
  }

  pub fn limit<T: MaybeOptional<i64>>(mut self, limit: T) -> Self {
    self.limit = limit.get_optional();
    self
  }

  pub fn list(self) -> Result<Vec<CommentReplyView>, Error> {
    use diesel::dsl::*;

    // The left join below will return None in this case
    let person_id_join = self.my_person_id.unwrap_or(PersonId(-1));

    let mut query = comment_reply::table
      .inner_join(comment::table)
      .inner_join(person::table.on(comment::creator_id.eq(person::id)))
      .inner_join(post::table.on(comment::post_id.eq(post::id)))
      .inner_join(community::table.on(post::community_id.eq(community::id)))
      .inner_join(person_alias_1::table)
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
        PersonAlias1::safe_columns_tuple(),
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

    query = match self.sort.unwrap_or(SortType::Hot) {
      SortType::Hot | SortType::Active => query
        .order_by(hot_rank(comment_aggregates::score, comment_aggregates::published).desc())
        .then_order_by(comment_aggregates::published.desc()),
      SortType::New | SortType::MostComments | SortType::NewComments => {
        query.order_by(comment::published.desc())
      }
      SortType::TopAll => query.order_by(comment_aggregates::score.desc()),
      SortType::TopYear => query
        .filter(comment::published.gt(now - 1.years()))
        .order_by(comment_aggregates::score.desc()),
      SortType::TopMonth => query
        .filter(comment::published.gt(now - 1.months()))
        .order_by(comment_aggregates::score.desc()),
      SortType::TopWeek => query
        .filter(comment::published.gt(now - 1.weeks()))
        .order_by(comment_aggregates::score.desc()),
      SortType::TopDay => query
        .filter(comment::published.gt(now - 1.days()))
        .order_by(comment_aggregates::score.desc()),
    };

    let (limit, offset) = limit_and_offset(self.page, self.limit)?;

    let res = query
      .limit(limit)
      .offset(offset)
      .load::<CommentReplyViewTuple>(self.conn)?;

    Ok(CommentReplyView::from_tuple_to_vec(res))
  }
}

impl ViewToVec for CommentReplyView {
  type DbTuple = CommentReplyViewTuple;
  fn from_tuple_to_vec(items: Vec<Self::DbTuple>) -> Vec<Self> {
    items
      .iter()
      .map(|a| Self {
        comment_reply: a.0.to_owned(),
        comment: a.1.to_owned(),
        creator: a.2.to_owned(),
        post: a.3.to_owned(),
        community: a.4.to_owned(),
        recipient: a.5.to_owned(),
        counts: a.6.to_owned(),
        creator_banned_from_community: a.7.is_some(),
        subscribed: CommunityFollower::to_subscribed_type(&a.8),
        saved: a.9.is_some(),
        creator_blocked: a.10.is_some(),
        my_vote: a.11,
      })
      .collect::<Vec<Self>>()
  }
}
