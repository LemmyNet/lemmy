use crate::{
  aggregates::community_aggregates::CommunityAggregates,
  functions::hot_rank,
  fuzzy_search,
  limit_and_offset,
  schema::{category, community, community_aggregates, community_follower, user_},
  source::{
    category::Category,
    community::{Community, CommunityFollower, CommunitySafe},
    user::{UserSafe, User_},
  },
  views::ViewToVec,
  MaybeOptional,
  SortType,
  ToSafe,
};
use diesel::{result::Error, *};
use serde::Serialize;

#[derive(Debug, Serialize, Clone)]
pub struct CommunityView {
  pub community: CommunitySafe,
  pub creator: UserSafe,
  pub category: Category,
  pub subscribed: bool,
  pub counts: CommunityAggregates,
}

type CommunityViewTuple = (
  CommunitySafe,
  UserSafe,
  Category,
  CommunityAggregates,
  Option<CommunityFollower>,
);

impl CommunityView {
  pub fn read(
    conn: &PgConnection,
    community_id: i32,
    my_user_id: Option<i32>,
  ) -> Result<Self, Error> {
    // The left join below will return None in this case
    let user_id_join = my_user_id.unwrap_or(-1);

    let (community, creator, category, counts, follower) = community::table
      .find(community_id)
      .inner_join(user_::table)
      .inner_join(category::table)
      .inner_join(community_aggregates::table)
      .left_join(
        community_follower::table.on(
          community::id
            .eq(community_follower::community_id)
            .and(community_follower::user_id.eq(user_id_join)),
        ),
      )
      .select((
        Community::safe_columns_tuple(),
        User_::safe_columns_tuple(),
        category::all_columns,
        community_aggregates::all_columns,
        community_follower::all_columns.nullable(),
      ))
      .first::<CommunityViewTuple>(conn)?;

    Ok(CommunityView {
      community,
      creator,
      category,
      subscribed: follower.is_some(),
      counts,
    })
  }
}

mod join_types {
  use crate::schema::{category, community, community_aggregates, community_follower, user_};
  use diesel::{
    pg::Pg,
    query_builder::BoxedSelectStatement,
    query_source::joins::{Inner, Join, JoinOn, LeftOuter},
    sql_types::*,
  };

  /// TODO awful, but necessary because of the boxed join
  pub(super) type BoxedCommunityJoin<'a> = BoxedSelectStatement<
    'a,
    (
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
      (Integer, Text),
      (Integer, Integer, BigInt, BigInt, BigInt),
      Nullable<(Integer, Integer, Integer, Timestamp, Nullable<Bool>)>,
    ),
    JoinOn<
      Join<
        JoinOn<
          Join<
            JoinOn<
              Join<
                JoinOn<
                  Join<community::table, user_::table, Inner>,
                  diesel::expression::operators::Eq<
                    diesel::expression::nullable::Nullable<community::columns::creator_id>,
                    diesel::expression::nullable::Nullable<user_::columns::id>,
                  >,
                >,
                category::table,
                Inner,
              >,
              diesel::expression::operators::Eq<
                diesel::expression::nullable::Nullable<community::columns::category_id>,
                diesel::expression::nullable::Nullable<category::columns::id>,
              >,
            >,
            community_aggregates::table,
            Inner,
          >,
          diesel::expression::operators::Eq<
            diesel::expression::nullable::Nullable<community_aggregates::columns::community_id>,
            diesel::expression::nullable::Nullable<community::columns::id>,
          >,
        >,
        community_follower::table,
        LeftOuter,
      >,
      diesel::expression::operators::And<
        diesel::expression::operators::Eq<
          community::columns::id,
          community_follower::columns::community_id,
        >,
        diesel::expression::operators::Eq<
          community_follower::columns::user_id,
          diesel::expression::bound::Bound<diesel::sql_types::Integer, i32>,
        >,
      >,
    >,
    Pg,
  >;
}

pub struct CommunityQueryBuilder<'a> {
  conn: &'a PgConnection,
  query: join_types::BoxedCommunityJoin<'a>,
  sort: &'a SortType,
  show_nsfw: bool,
  search_term: Option<String>,
  page: Option<i64>,
  limit: Option<i64>,
}

impl<'a> CommunityQueryBuilder<'a> {
  pub fn create(conn: &'a PgConnection, my_user_id: Option<i32>) -> Self {
    // The left join below will return None in this case
    let user_id_join = my_user_id.unwrap_or(-1);

    let query = community::table
      .inner_join(user_::table)
      .inner_join(category::table)
      .inner_join(community_aggregates::table)
      .left_join(
        community_follower::table.on(
          community::id
            .eq(community_follower::community_id)
            .and(community_follower::user_id.eq(user_id_join)),
        ),
      )
      .select((
        Community::safe_columns_tuple(),
        User_::safe_columns_tuple(),
        category::all_columns,
        community_aggregates::all_columns,
        community_follower::all_columns.nullable(),
      ))
      .into_boxed();

    CommunityQueryBuilder {
      conn,
      query,
      sort: &SortType::Hot,
      show_nsfw: true,
      search_term: None,
      page: None,
      limit: None,
    }
  }

  pub fn sort(mut self, sort: &'a SortType) -> Self {
    self.sort = sort;
    self
  }

  pub fn show_nsfw(mut self, show_nsfw: bool) -> Self {
    self.show_nsfw = show_nsfw;
    self
  }

  pub fn search_term<T: MaybeOptional<String>>(mut self, search_term: T) -> Self {
    self.search_term = search_term.get_optional();
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

  pub fn list(self) -> Result<Vec<CommunityView>, Error> {
    let mut query = self.query;

    if let Some(search_term) = self.search_term {
      let searcher = fuzzy_search(&search_term);
      query = query
        .filter(community::name.ilike(searcher.to_owned()))
        .or_filter(community::title.ilike(searcher.to_owned()))
        .or_filter(community::description.ilike(searcher));
    };

    match self.sort {
      SortType::New => query = query.order_by(community::published.desc()),
      SortType::TopAll => query = query.order_by(community_aggregates::subscribers.desc()),
      // Covers all other sorts, including hot
      _ => {
        query = query
          // TODO do custom sql function for hot_rank, make sure this works
          .order_by(hot_rank(community_aggregates::subscribers, community::published).desc())
          .then_order_by(community_aggregates::subscribers.desc())
      }
    };

    if !self.show_nsfw {
      query = query.filter(community::nsfw.eq(false));
    };

    let (limit, offset) = limit_and_offset(self.page, self.limit);
    let res = query
      .limit(limit)
      .offset(offset)
      .filter(community::removed.eq(false))
      .filter(community::deleted.eq(false))
      .load::<CommunityViewTuple>(self.conn)?;

    Ok(CommunityView::to_vec(res))
  }
}

impl ViewToVec for CommunityView {
  type DbTuple = CommunityViewTuple;
  fn to_vec(communities: Vec<Self::DbTuple>) -> Vec<Self> {
    communities
      .iter()
      .map(|a| Self {
        community: a.0.to_owned(),
        creator: a.1.to_owned(),
        category: a.2.to_owned(),
        counts: a.3.to_owned(),
        subscribed: a.4.is_some(),
      })
      .collect::<Vec<Self>>()
  }
}
