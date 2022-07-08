use crate::structs::PersonViewSafe;
use diesel::{dsl::*, result::Error, *};
use lemmy_db_schema::{
  aggregates::structs::PersonAggregates,
  newtypes::PersonId,
  schema::{person, person_aggregates},
  source::person::{Person, PersonSafe},
  traits::{MaybeOptional, ToSafe, ViewToVec},
  utils::{fuzzy_search, limit_and_offset},
  SortType,
};

type PersonViewSafeTuple = (PersonSafe, PersonAggregates);

impl PersonViewSafe {
  pub fn read(conn: &PgConnection, person_id: PersonId) -> Result<Self, Error> {
    let (person, counts) = person::table
      .find(person_id)
      .inner_join(person_aggregates::table)
      .select((Person::safe_columns_tuple(), person_aggregates::all_columns))
      .first::<PersonViewSafeTuple>(conn)?;
    Ok(Self { person, counts })
  }

  pub fn admins(conn: &PgConnection) -> Result<Vec<Self>, Error> {
    let admins = person::table
      .inner_join(person_aggregates::table)
      .select((Person::safe_columns_tuple(), person_aggregates::all_columns))
      .filter(person::admin.eq(true))
      .order_by(person::published)
      .load::<PersonViewSafeTuple>(conn)?;

    Ok(Self::from_tuple_to_vec(admins))
  }

  pub fn banned(conn: &PgConnection) -> Result<Vec<Self>, Error> {
    let banned = person::table
      .inner_join(person_aggregates::table)
      .select((Person::safe_columns_tuple(), person_aggregates::all_columns))
      .filter(
        person::banned.eq(true).and(
          person::ban_expires
            .is_null()
            .or(person::ban_expires.gt(now)),
        ),
      )
      .load::<PersonViewSafeTuple>(conn)?;

    Ok(Self::from_tuple_to_vec(banned))
  }
}

pub struct PersonQueryBuilder<'a> {
  conn: &'a PgConnection,
  sort: Option<SortType>,
  search_term: Option<String>,
  page: Option<i64>,
  limit: Option<i64>,
}

impl<'a> PersonQueryBuilder<'a> {
  pub fn create(conn: &'a PgConnection) -> Self {
    PersonQueryBuilder {
      conn,
      search_term: None,
      sort: None,
      page: None,
      limit: None,
    }
  }

  pub fn sort<T: MaybeOptional<SortType>>(mut self, sort: T) -> Self {
    self.sort = sort.get_optional();
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

  pub fn list(self) -> Result<Vec<PersonViewSafe>, Error> {
    let mut query = person::table
      .inner_join(person_aggregates::table)
      .select((Person::safe_columns_tuple(), person_aggregates::all_columns))
      .into_boxed();

    if let Some(search_term) = self.search_term {
      query = query.filter(person::name.ilike(fuzzy_search(&search_term)));
    }

    query = match self.sort.unwrap_or(SortType::Hot) {
      SortType::Hot => query
        .order_by(person_aggregates::comment_score.desc())
        .then_order_by(person::published.desc()),
      SortType::Active => query
        .order_by(person_aggregates::comment_score.desc())
        .then_order_by(person::published.desc()),
      SortType::New | SortType::MostComments | SortType::NewComments => {
        query.order_by(person::published.desc())
      }
      SortType::TopAll => query.order_by(person_aggregates::comment_score.desc()),
      SortType::TopYear => query
        .filter(person::published.gt(now - 1.years()))
        .order_by(person_aggregates::comment_score.desc()),
      SortType::TopMonth => query
        .filter(person::published.gt(now - 1.months()))
        .order_by(person_aggregates::comment_score.desc()),
      SortType::TopWeek => query
        .filter(person::published.gt(now - 1.weeks()))
        .order_by(person_aggregates::comment_score.desc()),
      SortType::TopDay => query
        .filter(person::published.gt(now - 1.days()))
        .order_by(person_aggregates::comment_score.desc()),
    };

    let (limit, offset) = limit_and_offset(self.page, self.limit)?;
    query = query.limit(limit).offset(offset);

    let res = query.load::<PersonViewSafeTuple>(self.conn)?;

    Ok(PersonViewSafe::from_tuple_to_vec(res))
  }
}

impl ViewToVec for PersonViewSafe {
  type DbTuple = PersonViewSafeTuple;
  fn from_tuple_to_vec(items: Vec<Self::DbTuple>) -> Vec<Self> {
    items
      .iter()
      .map(|a| Self {
        person: a.0.to_owned(),
        counts: a.1.to_owned(),
      })
      .collect::<Vec<Self>>()
  }
}
