use crate::{
  aggregates::comment_aggregates::CommentAggregates,
  functions::hot_rank,
  limit_and_offset,
  schema::{
    comment,
    comment_aggregates,
    comment_like,
    comment_saved,
    community,
    community_follower,
    community_user_ban,
    post,
    user_,
    user_alias_1,
    user_mention,
  },
  source::{
    comment::{Comment, CommentSaved},
    community::{Community, CommunityFollower, CommunitySafe, CommunityUserBan},
    post::Post,
    user::{UserAlias1, UserSafe, UserSafeAlias1, User_},
    user_mention::UserMention,
  },
  views::ViewToVec,
  MaybeOptional,
  SortType,
  ToSafe,
};
use diesel::{result::Error, *};
use serde::Serialize;

#[derive(Debug, PartialEq, Serialize, Clone)]
pub struct UserMentionView {
  pub user_mention: UserMention,
  pub comment: Comment,
  pub creator: UserSafe,
  pub post: Post,
  pub community: CommunitySafe,
  pub recipient: UserSafeAlias1,
  pub counts: CommentAggregates,
  pub creator_banned_from_community: bool, // Left Join to CommunityUserBan
  pub subscribed: bool,                    // Left join to CommunityFollower
  pub saved: bool,                         // Left join to CommentSaved
  pub my_vote: Option<i16>,                // Left join to CommentLike
}

type UserMentionViewTuple = (
  UserMention,
  Comment,
  UserSafe,
  Post,
  CommunitySafe,
  UserSafeAlias1,
  CommentAggregates,
  Option<CommunityUserBan>,
  Option<CommunityFollower>,
  Option<CommentSaved>,
  Option<i16>,
);

impl UserMentionView {
  pub fn read(
    conn: &PgConnection,
    user_mention_id: i32,
    my_user_id: Option<i32>,
  ) -> Result<Self, Error> {
    // The left join below will return None in this case
    let user_id_join = my_user_id.unwrap_or(-1);

    let (
      user_mention,
      comment,
      creator,
      post,
      community,
      recipient,
      counts,
      creator_banned_from_community,
      subscribed,
      saved,
      my_vote,
    ) = user_mention::table
      .find(user_mention_id)
      .inner_join(comment::table)
      .inner_join(user_::table.on(comment::creator_id.eq(user_::id)))
      .inner_join(post::table.on(comment::post_id.eq(post::id)))
      .inner_join(community::table.on(post::community_id.eq(community::id)))
      .inner_join(user_alias_1::table)
      .inner_join(comment_aggregates::table.on(comment::id.eq(comment_aggregates::comment_id)))
      .left_join(
        community_user_ban::table.on(
          community::id
            .eq(community_user_ban::community_id)
            .and(community_user_ban::user_id.eq(comment::creator_id)),
        ),
      )
      .left_join(
        community_follower::table.on(
          post::community_id
            .eq(community_follower::community_id)
            .and(community_follower::user_id.eq(user_id_join)),
        ),
      )
      .left_join(
        comment_saved::table.on(
          comment::id
            .eq(comment_saved::comment_id)
            .and(comment_saved::user_id.eq(user_id_join)),
        ),
      )
      .left_join(
        comment_like::table.on(
          comment::id
            .eq(comment_like::comment_id)
            .and(comment_like::user_id.eq(user_id_join)),
        ),
      )
      .select((
        user_mention::all_columns,
        comment::all_columns,
        User_::safe_columns_tuple(),
        post::all_columns,
        Community::safe_columns_tuple(),
        UserAlias1::safe_columns_tuple(),
        comment_aggregates::all_columns,
        community_user_ban::all_columns.nullable(),
        community_follower::all_columns.nullable(),
        comment_saved::all_columns.nullable(),
        comment_like::score.nullable(),
      ))
      .first::<UserMentionViewTuple>(conn)?;

    Ok(UserMentionView {
      user_mention,
      comment,
      creator,
      post,
      community,
      recipient,
      counts,
      creator_banned_from_community: creator_banned_from_community.is_some(),
      subscribed: subscribed.is_some(),
      saved: saved.is_some(),
      my_vote,
    })
  }
}

mod join_types {
  use crate::schema::{
    comment,
    comment_aggregates,
    comment_like,
    comment_saved,
    community,
    community_follower,
    community_user_ban,
    post,
    user_,
    user_alias_1,
    user_mention,
  };
  use diesel::{
    pg::Pg,
    query_builder::BoxedSelectStatement,
    query_source::joins::{Inner, Join, JoinOn, LeftOuter},
    sql_types::*,
  };

  // /// TODO awful, but necessary because of the boxed join
  pub(super) type BoxedUserMentionJoin<'a> = BoxedSelectStatement<
    'a,
    (
      (Integer, Integer, Integer, Bool, Timestamp),
      (
        Integer,
        Integer,
        Integer,
        Nullable<Integer>,
        Text,
        Bool,
        Bool,
        Timestamp,
        Nullable<Timestamp>,
        Bool,
        Text,
        Bool,
      ),
      (
        Integer,
        Text,
        Nullable<Text>,
        Nullable<Text>,
        Bool,
        Bool,
        Timestamp,
        Nullable<Timestamp>,
        Nullable<Text>,
        Text,
        Nullable<Text>,
        Bool,
        Nullable<Text>,
        Bool,
      ),
      (
        Integer,
        Text,
        Nullable<Text>,
        Nullable<Text>,
        Integer,
        Integer,
        Bool,
        Bool,
        Timestamp,
        Nullable<Timestamp>,
        Bool,
        Bool,
        Bool,
        Nullable<Text>,
        Nullable<Text>,
        Nullable<Text>,
        Nullable<Text>,
        Text,
        Bool,
      ),
      (
        Integer,
        Text,
        Text,
        Nullable<Text>,
        Integer,
        Integer,
        Bool,
        Timestamp,
        Nullable<Timestamp>,
        Bool,
        Bool,
        Text,
        Bool,
        Nullable<Text>,
        Nullable<Text>,
      ),
      (
        Integer,
        Text,
        Nullable<Text>,
        Nullable<Text>,
        Bool,
        Bool,
        Timestamp,
        Nullable<Timestamp>,
        Nullable<Text>,
        Text,
        Nullable<Text>,
        Bool,
        Nullable<Text>,
        Bool,
      ),
      (Integer, Integer, BigInt, BigInt, BigInt),
      Nullable<(Integer, Integer, Integer, Timestamp)>,
      Nullable<(Integer, Integer, Integer, Timestamp, Nullable<Bool>)>,
      Nullable<(Integer, Integer, Integer, Timestamp)>,
      Nullable<SmallInt>,
    ),
    JoinOn<
      Join<
        JoinOn<
          Join<
            JoinOn<
              Join<
                JoinOn<
                  Join<
                    JoinOn<
                      Join<
                        JoinOn<
                          Join<
                            JoinOn<
                              Join<
                                JoinOn<
                                  Join<
                                    JoinOn<
                                      Join<
                                        JoinOn<
                                          Join<user_mention::table, comment::table, Inner>,
                                          diesel::expression::operators::Eq<
                                            diesel::expression::nullable::Nullable<
                                              user_mention::columns::comment_id,
                                            >,
                                            diesel::expression::nullable::Nullable<
                                              comment::columns::id,
                                            >,
                                          >,
                                        >,
                                        user_::table,
                                        Inner,
                                      >,
                                      diesel::expression::operators::Eq<
                                        comment::columns::creator_id,
                                        user_::columns::id,
                                      >,
                                    >,
                                    post::table,
                                    Inner,
                                  >,
                                  diesel::expression::operators::Eq<
                                    comment::columns::post_id,
                                    post::columns::id,
                                  >,
                                >,
                                community::table,
                                Inner,
                              >,
                              diesel::expression::operators::Eq<
                                post::columns::community_id,
                                community::columns::id,
                              >,
                            >,
                            user_alias_1::table,
                            Inner,
                          >,
                          diesel::expression::operators::Eq<
                            diesel::expression::nullable::Nullable<
                              user_mention::columns::recipient_id,
                            >,
                            diesel::expression::nullable::Nullable<user_alias_1::columns::id>,
                          >,
                        >,
                        comment_aggregates::table,
                        Inner,
                      >,
                      diesel::expression::operators::Eq<
                        comment::columns::id,
                        comment_aggregates::columns::comment_id,
                      >,
                    >,
                    community_user_ban::table,
                    LeftOuter,
                  >,
                  diesel::expression::operators::And<
                    diesel::expression::operators::Eq<
                      community::columns::id,
                      community_user_ban::columns::community_id,
                    >,
                    diesel::expression::operators::Eq<
                      community_user_ban::columns::user_id,
                      comment::columns::creator_id,
                    >,
                  >,
                >,
                community_follower::table,
                LeftOuter,
              >,
              diesel::expression::operators::And<
                diesel::expression::operators::Eq<
                  post::columns::community_id,
                  community_follower::columns::community_id,
                >,
                diesel::expression::operators::Eq<
                  community_follower::columns::user_id,
                  diesel::expression::bound::Bound<Integer, i32>,
                >,
              >,
            >,
            comment_saved::table,
            LeftOuter,
          >,
          diesel::expression::operators::And<
            diesel::expression::operators::Eq<
              comment::columns::id,
              comment_saved::columns::comment_id,
            >,
            diesel::expression::operators::Eq<
              comment_saved::columns::user_id,
              diesel::expression::bound::Bound<Integer, i32>,
            >,
          >,
        >,
        comment_like::table,
        LeftOuter,
      >,
      diesel::expression::operators::And<
        diesel::expression::operators::Eq<comment::columns::id, comment_like::columns::comment_id>,
        diesel::expression::operators::Eq<
          comment_like::columns::user_id,
          diesel::expression::bound::Bound<Integer, i32>,
        >,
      >,
    >,
    Pg,
  >;
}

pub struct UserMentionQueryBuilder<'a> {
  conn: &'a PgConnection,
  query: join_types::BoxedUserMentionJoin<'a>,
  for_recipient_id: i32,
  sort: &'a SortType,
  unread_only: bool,
  page: Option<i64>,
  limit: Option<i64>,
}

impl<'a> UserMentionQueryBuilder<'a> {
  pub fn create(conn: &'a PgConnection, my_user_id: Option<i32>, for_recipient_id: i32) -> Self {
    // The left join below will return None in this case
    let user_id_join = my_user_id.unwrap_or(-1);

    let query = user_mention::table
      .inner_join(comment::table)
      .inner_join(user_::table.on(comment::creator_id.eq(user_::id)))
      .inner_join(post::table.on(comment::post_id.eq(post::id)))
      .inner_join(community::table.on(post::community_id.eq(community::id)))
      .inner_join(user_alias_1::table)
      .inner_join(comment_aggregates::table.on(comment::id.eq(comment_aggregates::comment_id)))
      .left_join(
        community_user_ban::table.on(
          community::id
            .eq(community_user_ban::community_id)
            .and(community_user_ban::user_id.eq(comment::creator_id)),
        ),
      )
      .left_join(
        community_follower::table.on(
          post::community_id
            .eq(community_follower::community_id)
            .and(community_follower::user_id.eq(user_id_join)),
        ),
      )
      .left_join(
        comment_saved::table.on(
          comment::id
            .eq(comment_saved::comment_id)
            .and(comment_saved::user_id.eq(user_id_join)),
        ),
      )
      .left_join(
        comment_like::table.on(
          comment::id
            .eq(comment_like::comment_id)
            .and(comment_like::user_id.eq(user_id_join)),
        ),
      )
      .select((
        user_mention::all_columns,
        comment::all_columns,
        User_::safe_columns_tuple(),
        post::all_columns,
        Community::safe_columns_tuple(),
        UserAlias1::safe_columns_tuple(),
        comment_aggregates::all_columns,
        community_user_ban::all_columns.nullable(),
        community_follower::all_columns.nullable(),
        comment_saved::all_columns.nullable(),
        comment_like::score.nullable(),
      ))
      .into_boxed();

    UserMentionQueryBuilder {
      conn,
      query,
      for_recipient_id,
      sort: &SortType::New,
      unread_only: false,
      page: None,
      limit: None,
    }
  }

  pub fn sort(mut self, sort: &'a SortType) -> Self {
    self.sort = sort;
    self
  }

  pub fn unread_only(mut self, unread_only: bool) -> Self {
    self.unread_only = unread_only;
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

  pub fn list(self) -> Result<Vec<UserMentionView>, Error> {
    use diesel::dsl::*;

    let mut query = self.query;

    query = query.filter(user_mention::recipient_id.eq(self.for_recipient_id));

    if self.unread_only {
      query = query.filter(user_mention::read.eq(false));
    }

    query = match self.sort {
      SortType::Hot | SortType::Active => query
        .order_by(hot_rank(comment_aggregates::score, comment::published).desc())
        .then_order_by(comment::published.desc()),
      SortType::New => query.order_by(comment::published.desc()),
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

    let (limit, offset) = limit_and_offset(self.page, self.limit);

    let res = query
      .limit(limit)
      .offset(offset)
      .load::<UserMentionViewTuple>(self.conn)?;

    Ok(UserMentionView::to_vec(res))
  }
}

impl ViewToVec for UserMentionView {
  type DbTuple = UserMentionViewTuple;
  fn to_vec(posts: Vec<Self::DbTuple>) -> Vec<Self> {
    posts
      .iter()
      .map(|a| Self {
        user_mention: a.0.to_owned(),
        comment: a.1.to_owned(),
        creator: a.2.to_owned(),
        post: a.3.to_owned(),
        community: a.4.to_owned(),
        recipient: a.5.to_owned(),
        counts: a.6.to_owned(),
        creator_banned_from_community: a.7.is_some(),
        subscribed: a.8.is_some(),
        saved: a.9.is_some(),
        my_vote: a.10,
      })
      .collect::<Vec<Self>>()
  }
}
