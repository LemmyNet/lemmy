use crate::{
  aggregates::post_aggregates::PostAggregates,
  community::{Community, CommunityFollower, CommunitySafe, CommunityUserBan},
  functions::hot_rank,
  fuzzy_search,
  limit_and_offset,
  post::{Post, PostRead, PostSaved},
  schema::{
    community,
    community_follower,
    community_user_ban,
    post,
    post_aggregates,
    post_like,
    post_read,
    post_saved,
    user_,
  },
  user::{UserSafe, User_},
  views::ViewToVec,
  ListingType,
  MaybeOptional,
  SortType,
  ToSafe,
};
use diesel::{result::Error, *};
use serde::Serialize;

#[derive(Debug, Serialize, Clone)]
pub struct PostView {
  pub post: Post,
  pub creator: UserSafe,
  pub community: CommunitySafe,
  pub counts: PostAggregates,
  pub subscribed: bool,            // Left join to CommunityFollower
  pub banned_from_community: bool, // Left Join to CommunityUserBan
  pub saved: bool,                 // Left join to PostSaved
  pub read: bool,                  // Left join to PostRead
  pub my_vote: Option<i16>,        // Left join to PostLike
}

type PostViewTuple = (
  Post,
  UserSafe,
  CommunitySafe,
  PostAggregates,
  Option<CommunityFollower>,
  Option<CommunityUserBan>,
  Option<PostSaved>,
  Option<PostRead>,
  Option<i16>,
);

impl PostView {
  pub fn read(conn: &PgConnection, post_id: i32, my_user_id: Option<i32>) -> Result<Self, Error> {
    // The left join below will return None in this case
    let user_id_join = my_user_id.unwrap_or(-1);

    let (post, creator, community, counts, follower, banned_from_community, saved, read, my_vote) =
      post::table
        .find(post_id)
        .inner_join(user_::table)
        .inner_join(community::table)
        .inner_join(post_aggregates::table)
        .left_join(
          community_follower::table.on(
            post::community_id
              .eq(community_follower::community_id)
              .and(community_follower::user_id.eq(user_id_join)),
          ),
        )
        .left_join(
          community_user_ban::table.on(
            post::community_id
              .eq(community_user_ban::community_id)
              .and(community_user_ban::user_id.eq(user_id_join)),
          ),
        )
        .left_join(
          post_saved::table.on(
            post::id
              .eq(post_saved::post_id)
              .and(post_saved::user_id.eq(user_id_join)),
          ),
        )
        .left_join(
          post_read::table.on(
            post::id
              .eq(post_read::post_id)
              .and(post_read::user_id.eq(user_id_join)),
          ),
        )
        .left_join(
          post_like::table.on(
            post::id
              .eq(post_like::post_id)
              .and(post_like::user_id.eq(user_id_join)),
          ),
        )
        .select((
          post::all_columns,
          User_::safe_columns_tuple(),
          Community::safe_columns_tuple(),
          post_aggregates::all_columns,
          community_follower::all_columns.nullable(),
          community_user_ban::all_columns.nullable(),
          post_saved::all_columns.nullable(),
          post_read::all_columns.nullable(),
          post_like::score.nullable(),
        ))
        .first::<PostViewTuple>(conn)?;

    Ok(PostView {
      post,
      creator,
      community,
      counts,
      subscribed: follower.is_some(),
      banned_from_community: banned_from_community.is_some(),
      saved: saved.is_some(),
      read: read.is_some(),
      my_vote,
    })
  }
}

mod join_types {
  use crate::schema::{
    community,
    community_follower,
    community_user_ban,
    post,
    post_aggregates,
    post_like,
    post_read,
    post_saved,
    user_,
  };
  use diesel::{
    pg::Pg,
    query_builder::BoxedSelectStatement,
    query_source::joins::{Inner, Join, JoinOn, LeftOuter},
    sql_types::*,
  };

  /// TODO awful, but necessary because of the boxed join
  pub(super) type BoxedPostJoin<'a> = BoxedSelectStatement<
    'a,
    (
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
      (Integer, Integer, BigInt, BigInt, BigInt, BigInt, Timestamp),
      Nullable<(Integer, Integer, Integer, Timestamp, Nullable<Bool>)>,
      Nullable<(Integer, Integer, Integer, Timestamp)>,
      Nullable<(Integer, Integer, Integer, Timestamp)>,
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
                                  Join<post::table, user_::table, Inner>,
                                  diesel::expression::operators::Eq<
                                    diesel::expression::nullable::Nullable<
                                      post::columns::creator_id,
                                    >,
                                    diesel::expression::nullable::Nullable<user_::columns::id>,
                                  >,
                                >,
                                community::table,
                                Inner,
                              >,
                              diesel::expression::operators::Eq<
                                diesel::expression::nullable::Nullable<post::columns::community_id>,
                                diesel::expression::nullable::Nullable<community::columns::id>,
                              >,
                            >,
                            post_aggregates::table,
                            Inner,
                          >,
                          diesel::expression::operators::Eq<
                            diesel::expression::nullable::Nullable<
                              post_aggregates::columns::post_id,
                            >,
                            diesel::expression::nullable::Nullable<post::columns::id>,
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
                          diesel::expression::bound::Bound<diesel::sql_types::Integer, i32>,
                        >,
                      >,
                    >,
                    community_user_ban::table,
                    LeftOuter,
                  >,
                  diesel::expression::operators::And<
                    diesel::expression::operators::Eq<
                      post::columns::community_id,
                      community_user_ban::columns::community_id,
                    >,
                    diesel::expression::operators::Eq<
                      community_user_ban::columns::user_id,
                      diesel::expression::bound::Bound<diesel::sql_types::Integer, i32>,
                    >,
                  >,
                >,
                post_saved::table,
                LeftOuter,
              >,
              diesel::expression::operators::And<
                diesel::expression::operators::Eq<post::columns::id, post_saved::columns::post_id>,
                diesel::expression::operators::Eq<
                  post_saved::columns::user_id,
                  diesel::expression::bound::Bound<diesel::sql_types::Integer, i32>,
                >,
              >,
            >,
            post_read::table,
            LeftOuter,
          >,
          diesel::expression::operators::And<
            diesel::expression::operators::Eq<post::columns::id, post_read::columns::post_id>,
            diesel::expression::operators::Eq<
              post_read::columns::user_id,
              diesel::expression::bound::Bound<diesel::sql_types::Integer, i32>,
            >,
          >,
        >,
        post_like::table,
        LeftOuter,
      >,
      diesel::expression::operators::And<
        diesel::expression::operators::Eq<post::columns::id, post_like::columns::post_id>,
        diesel::expression::operators::Eq<
          post_like::columns::user_id,
          diesel::expression::bound::Bound<diesel::sql_types::Integer, i32>,
        >,
      >,
    >,
    Pg,
  >;
}

pub struct PostQueryBuilder<'a> {
  conn: &'a PgConnection,
  query: join_types::BoxedPostJoin<'a>,
  listing_type: &'a ListingType,
  sort: &'a SortType,
  for_creator_id: Option<i32>,
  for_community_id: Option<i32>,
  for_community_name: Option<String>,
  search_term: Option<String>,
  url_search: Option<String>,
  show_nsfw: bool,
  saved_only: bool,
  unread_only: bool,
  page: Option<i64>,
  limit: Option<i64>,
}

impl<'a> PostQueryBuilder<'a> {
  pub fn create(conn: &'a PgConnection, my_user_id: Option<i32>) -> Self {
    // The left join below will return None in this case
    let user_id_join = my_user_id.unwrap_or(-1);

    let query = post::table
      .inner_join(user_::table)
      .inner_join(community::table)
      .inner_join(post_aggregates::table)
      .left_join(
        community_follower::table.on(
          post::community_id
            .eq(community_follower::community_id)
            .and(community_follower::user_id.eq(user_id_join)),
        ),
      )
      .left_join(
        community_user_ban::table.on(
          post::community_id
            .eq(community_user_ban::community_id)
            .and(community_user_ban::user_id.eq(user_id_join)),
        ),
      )
      .left_join(
        post_saved::table.on(
          post::id
            .eq(post_saved::post_id)
            .and(post_saved::user_id.eq(user_id_join)),
        ),
      )
      .left_join(
        post_read::table.on(
          post::id
            .eq(post_read::post_id)
            .and(post_read::user_id.eq(user_id_join)),
        ),
      )
      .left_join(
        post_like::table.on(
          post::id
            .eq(post_like::post_id)
            .and(post_like::user_id.eq(user_id_join)),
        ),
      )
      .select((
        post::all_columns,
        User_::safe_columns_tuple(),
        Community::safe_columns_tuple(),
        post_aggregates::all_columns,
        community_follower::all_columns.nullable(),
        community_user_ban::all_columns.nullable(),
        post_saved::all_columns.nullable(),
        post_read::all_columns.nullable(),
        post_like::score.nullable(),
      ))
      .into_boxed();

    PostQueryBuilder {
      conn,
      query,
      listing_type: &ListingType::All,
      sort: &SortType::Hot,
      for_creator_id: None,
      for_community_id: None,
      for_community_name: None,
      search_term: None,
      url_search: None,
      show_nsfw: true,
      saved_only: false,
      unread_only: false,
      page: None,
      limit: None,
    }
  }

  pub fn listing_type(mut self, listing_type: &'a ListingType) -> Self {
    self.listing_type = listing_type;
    self
  }

  pub fn sort(mut self, sort: &'a SortType) -> Self {
    self.sort = sort;
    self
  }

  pub fn for_community_id<T: MaybeOptional<i32>>(mut self, for_community_id: T) -> Self {
    self.for_community_id = for_community_id.get_optional();
    self
  }

  pub fn for_community_name<T: MaybeOptional<String>>(mut self, for_community_name: T) -> Self {
    self.for_community_name = for_community_name.get_optional();
    self
  }

  pub fn for_creator_id<T: MaybeOptional<i32>>(mut self, for_creator_id: T) -> Self {
    self.for_creator_id = for_creator_id.get_optional();
    self
  }

  pub fn search_term<T: MaybeOptional<String>>(mut self, search_term: T) -> Self {
    self.search_term = search_term.get_optional();
    self
  }

  pub fn url_search<T: MaybeOptional<String>>(mut self, url_search: T) -> Self {
    self.url_search = url_search.get_optional();
    self
  }

  pub fn show_nsfw(mut self, show_nsfw: bool) -> Self {
    self.show_nsfw = show_nsfw;
    self
  }

  pub fn saved_only(mut self, saved_only: bool) -> Self {
    self.saved_only = saved_only;
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

  pub fn list(self) -> Result<Vec<PostView>, Error> {
    use diesel::dsl::*;

    let mut query = self.query;

    query = match self.listing_type {
      ListingType::Subscribed => query.filter(community_follower::user_id.is_not_null()), // TODO could be this: and(community_follower::user_id.eq(user_id_join)),
      ListingType::Local => query.filter(community::local.eq(true)),
      _ => query,
    };

    if let Some(for_community_id) = self.for_community_id {
      query = query
        .filter(post::community_id.eq(for_community_id))
        .then_order_by(post::stickied.desc());
    }

    if let Some(for_community_name) = self.for_community_name {
      query = query
        .filter(community::name.eq(for_community_name))
        .filter(community::local.eq(true))
        .then_order_by(post::stickied.desc());
    }

    if let Some(url_search) = self.url_search {
      query = query.filter(post::url.eq(url_search));
    }

    if let Some(search_term) = self.search_term {
      let searcher = fuzzy_search(&search_term);
      query = query.filter(
        post::name
          .ilike(searcher.to_owned())
          .or(post::body.ilike(searcher)),
      );
    }

    query = match self.sort {
      SortType::Active => query
        .then_order_by(
          hot_rank(post_aggregates::score, post_aggregates::newest_comment_time).desc(),
        )
        .then_order_by(post::published.desc()),
      SortType::Hot => query
        .then_order_by(hot_rank(post_aggregates::score, post::published).desc())
        .then_order_by(post::published.desc()),
      SortType::New => query.then_order_by(post::published.desc()),
      SortType::TopAll => query.then_order_by(post_aggregates::score.desc()),
      SortType::TopYear => query
        .filter(post::published.gt(now - 1.years()))
        .then_order_by(post_aggregates::score.desc()),
      SortType::TopMonth => query
        .filter(post::published.gt(now - 1.months()))
        .then_order_by(post_aggregates::score.desc()),
      SortType::TopWeek => query
        .filter(post::published.gt(now - 1.weeks()))
        .then_order_by(post_aggregates::score.desc()),
      SortType::TopDay => query
        .filter(post::published.gt(now - 1.days()))
        .then_order_by(post_aggregates::score.desc()),
    };

    // If its for a specific user, show the removed / deleted
    if let Some(for_creator_id) = self.for_creator_id {
      query = query.filter(post::creator_id.eq(for_creator_id));
    }

    if !self.show_nsfw {
      query = query
        .filter(post::nsfw.eq(false))
        .filter(community::nsfw.eq(false));
    };

    // TODO  These two might be wrong
    if self.saved_only {
      query = query.filter(post_saved::id.is_not_null());
    };

    if self.unread_only {
      query = query.filter(post_read::id.is_not_null());
    };

    let (limit, offset) = limit_and_offset(self.page, self.limit);

    let res = query
      .limit(limit)
      .offset(offset)
      .filter(post::removed.eq(false))
      .filter(post::deleted.eq(false))
      .filter(community::removed.eq(false))
      .filter(community::deleted.eq(false))
      .load::<PostViewTuple>(self.conn)?;

    Ok(PostView::to_vec(res))
  }
}

impl ViewToVec for PostView {
  type DbTuple = PostViewTuple;
  fn to_vec(posts: Vec<Self::DbTuple>) -> Vec<Self> {
    posts
      .iter()
      .map(|a| Self {
        post: a.0.to_owned(),
        creator: a.1.to_owned(),
        community: a.2.to_owned(),
        counts: a.3.to_owned(),
        subscribed: a.4.is_some(),
        banned_from_community: a.5.is_some(),
        saved: a.6.is_some(),
        read: a.7.is_some(),
        my_vote: a.8,
      })
      .collect::<Vec<Self>>()
  }
}
