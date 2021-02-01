use diesel::{dsl::*, result::Error, *};
use lemmy_db_queries::{
  aggregates::user_aggregates::UserAggregates,
  fuzzy_search,
  limit_and_offset,
  MaybeOptional,
  SortType,
  ToSafe,
  ViewToVec,
};
use lemmy_db_schema::{
  schema::{user_, user_aggregates},
  source::user::{UserSafe, User_},
};
use serde::Serialize;

#[derive(Debug, Serialize, Clone)]
pub struct UserViewSafe {
  pub user: UserSafe,
  pub counts: UserAggregates,
}

type UserViewSafeTuple = (UserSafe, UserAggregates);

impl UserViewSafe {
  pub fn read(conn: &PgConnection, id: i32) -> Result<Self, Error> {
    let (user, counts) = user_::table
      .find(id)
      .inner_join(user_aggregates::table)
      .select((User_::safe_columns_tuple(), user_aggregates::all_columns))
      .first::<UserViewSafeTuple>(conn)?;
    Ok(Self { user, counts })
  }

  pub fn admins(conn: &PgConnection) -> Result<Vec<Self>, Error> {
    let admins = user_::table
      .inner_join(user_aggregates::table)
      .select((User_::safe_columns_tuple(), user_aggregates::all_columns))
      .filter(user_::admin.eq(true))
      .order_by(user_::published)
      .load::<UserViewSafeTuple>(conn)?;

    Ok(Self::from_tuple_to_vec(admins))
  }

  pub fn banned(conn: &PgConnection) -> Result<Vec<Self>, Error> {
    let banned = user_::table
      .inner_join(user_aggregates::table)
      .select((User_::safe_columns_tuple(), user_aggregates::all_columns))
      .filter(user_::banned.eq(true))
      .load::<UserViewSafeTuple>(conn)?;

    Ok(Self::from_tuple_to_vec(banned))
  }
}

pub struct UserQueryBuilder<'a> {
  conn: &'a PgConnection,
  sort: &'a SortType,
  search_term: Option<String>,
  page: Option<i64>,
  limit: Option<i64>,
}

impl<'a> UserQueryBuilder<'a> {
  pub fn create(conn: &'a PgConnection) -> Self {
    UserQueryBuilder {
      conn,
      search_term: None,
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

  pub fn list(self) -> Result<Vec<UserViewSafe>, Error> {
    let mut query = user_::table
      .inner_join(user_aggregates::table)
      .select((User_::safe_columns_tuple(), user_aggregates::all_columns))
      .into_boxed();

    if let Some(search_term) = self.search_term {
      query = query.filter(user_::name.ilike(fuzzy_search(&search_term)));
    }

    query = match self.sort {
      SortType::Hot => query
        .order_by(user_aggregates::comment_score.desc())
        .then_order_by(user_::published.desc()),
      SortType::Active => query
        .order_by(user_aggregates::comment_score.desc())
        .then_order_by(user_::published.desc()),
      SortType::New | SortType::MostComments => query.order_by(user_::published.desc()),
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

    let res = query.load::<UserViewSafeTuple>(self.conn)?;

    Ok(UserViewSafe::from_tuple_to_vec(res))
  }
}

impl ViewToVec for UserViewSafe {
  type DbTuple = UserViewSafeTuple;
  fn from_tuple_to_vec(items: Vec<Self::DbTuple>) -> Vec<Self> {
    items
      .iter()
      .map(|a| Self {
        user: a.0.to_owned(),
        counts: a.1.to_owned(),
      })
      .collect::<Vec<Self>>()
  }
}
