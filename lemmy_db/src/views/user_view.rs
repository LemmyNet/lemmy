use crate::{
  aggregates::user_aggregates::UserAggregates,
  fuzzy_search,
  limit_and_offset,
  schema::{user_, user_aggregates},
  user::{UserSafe, User_},
  MaybeOptional,
  SortType,
  ToSafe,
};
use diesel::{dsl::*, result::Error, *};
use serde::Serialize;

#[derive(Debug, Serialize, Clone)]
pub struct UserViewSafe {
  pub user: UserSafe,
  pub counts: UserAggregates,
}

#[derive(Debug, Serialize, Clone)]
pub struct UserViewDangerous {
  pub user: User_,
  pub counts: UserAggregates,
}

impl UserViewDangerous {
  pub fn read(conn: &PgConnection, id: i32) -> Result<Self, Error> {
    let (user, counts) = user_::table
      .find(id)
      .inner_join(user_aggregates::table)
      .first::<(User_, UserAggregates)>(conn)?;
    Ok(Self { user, counts })
  }
}

impl UserViewSafe {
  pub fn read(conn: &PgConnection, id: i32) -> Result<Self, Error> {
    let (user, counts) = user_::table
      .find(id)
      .inner_join(user_aggregates::table)
      .select((User_::safe_columns_tuple(), user_aggregates::all_columns))
      .first::<(UserSafe, UserAggregates)>(conn)?;
    Ok(Self { user, counts })
  }

  pub fn admins(conn: &PgConnection) -> Result<Vec<Self>, Error> {
    let admins = user_::table
      .inner_join(user_aggregates::table)
      .select((User_::safe_columns_tuple(), user_aggregates::all_columns))
      .filter(user_::admin.eq(true))
      .order_by(user_::published)
      .load::<(UserSafe, UserAggregates)>(conn)?;

    Ok(to_vec(admins))
  }

  pub fn banned(conn: &PgConnection) -> Result<Vec<Self>, Error> {
    let banned = user_::table
      .inner_join(user_aggregates::table)
      .select((User_::safe_columns_tuple(), user_aggregates::all_columns))
      .filter(user_::banned.eq(true))
      .load::<(UserSafe, UserAggregates)>(conn)?;

    Ok(to_vec(banned))
  }
}

mod join_types {
  use crate::schema::{user_, user_aggregates};
  use diesel::{
    pg::Pg,
    query_builder::BoxedSelectStatement,
    query_source::joins::{Inner, Join, JoinOn},
    sql_types::*,
  };

  /// TODO awful, but necessary because of the boxed join
  pub(super) type BoxedUserJoin<'a> = BoxedSelectStatement<
    'a,
    (
      // UserSafe column types
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
      // UserAggregates column types
      (Integer, Integer, BigInt, BigInt, BigInt, BigInt),
    ),
    JoinOn<
      Join<user_::table, user_aggregates::table, Inner>,
      diesel::expression::operators::Eq<
        diesel::expression::nullable::Nullable<user_aggregates::columns::user_id>,
        diesel::expression::nullable::Nullable<user_::columns::id>,
      >,
    >,
    Pg,
  >;
}

pub struct UserQueryBuilder<'a> {
  conn: &'a PgConnection,
  query: join_types::BoxedUserJoin<'a>,
  sort: &'a SortType,
  page: Option<i64>,
  limit: Option<i64>,
}

impl<'a> UserQueryBuilder<'a> {
  pub fn create(conn: &'a PgConnection) -> Self {
    let query = user_::table
      .inner_join(user_aggregates::table)
      .select((User_::safe_columns_tuple(), user_aggregates::all_columns))
      .into_boxed();

    UserQueryBuilder {
      conn,
      query,
      sort: &SortType::Hot,
      page: None,
      limit: None,
    }
  }

  pub fn sort(mut self, sort: &'a SortType) -> Self {
    self.sort = sort;
    self
  }

  pub fn search_term<T: MaybeOptional<String>>(mut self, search_term: T) -> Self {
    if let Some(search_term) = search_term.get_optional() {
      self.query = self
        .query
        .filter(user_::name.ilike(fuzzy_search(&search_term)));
    }
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

  pub fn list(self) -> Result<Vec<UserViewSafe>, Error> {
    let mut query = self.query;

    query = match self.sort {
      SortType::Hot => query
        .order_by(user_aggregates::comment_score.desc())
        .then_order_by(user_::published.desc()),
      SortType::Active => query
        .order_by(user_aggregates::comment_score.desc())
        .then_order_by(user_::published.desc()),
      SortType::New => query.order_by(user_::published.desc()),
      SortType::TopAll => query.order_by(user_aggregates::comment_score.desc()),
      SortType::TopYear => query
        .filter(user_::published.gt(now - 1.years()))
        .order_by(user_aggregates::comment_score.desc()),
      SortType::TopMonth => query
        .filter(user_::published.gt(now - 1.months()))
        .order_by(user_aggregates::comment_score.desc()),
      SortType::TopWeek => query
        .filter(user_::published.gt(now - 1.weeks()))
        .order_by(user_aggregates::comment_score.desc()),
      SortType::TopDay => query
        .filter(user_::published.gt(now - 1.days()))
        .order_by(user_aggregates::comment_score.desc()),
    };

    let (limit, offset) = limit_and_offset(self.page, self.limit);
    query = query.limit(limit).offset(offset);

    let res = query.load::<(UserSafe, UserAggregates)>(self.conn)?;

    Ok(to_vec(res))
  }
}

fn to_vec(users: Vec<(UserSafe, UserAggregates)>) -> Vec<UserViewSafe> {
  users
    .iter()
    .map(|a| UserViewSafe {
      user: a.0.to_owned(),
      counts: a.1.to_owned(),
    })
    .collect::<Vec<UserViewSafe>>()
}
