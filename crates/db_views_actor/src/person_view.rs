use crate::structs::PersonViewSafe;
use diesel::{
  dsl::{now, IntervalDsl},
  result::Error,
  BoolExpressionMethods,
  ExpressionMethods,
  PgTextExpressionMethods,
  QueryDsl,
};
use diesel_async::RunQueryDsl;
use lemmy_db_schema::{
  aggregates::structs::PersonAggregates,
  newtypes::PersonId,
  schema::{person, person_aggregates},
  source::person::{Person, PersonSafe},
  traits::{ToSafe, ViewToVec},
  utils::{fuzzy_search, get_conn, limit_and_offset, DbPool},
  SortType,
};
use std::iter::Iterator;
use typed_builder::TypedBuilder;

type PersonViewSafeTuple = (PersonSafe, PersonAggregates);

impl PersonViewSafe {
  pub async fn read(pool: &DbPool, person_id: PersonId) -> Result<Self, Error> {
    let conn = &mut get_conn(pool).await?;
    let (person, counts) = person::table
      .find(person_id)
      .inner_join(person_aggregates::table)
      .select((Person::safe_columns_tuple(), person_aggregates::all_columns))
      .first::<PersonViewSafeTuple>(conn)
      .await?;
    Ok(Self { person, counts })
  }

  pub async fn admins(pool: &DbPool) -> Result<Vec<Self>, Error> {
    let conn = &mut get_conn(pool).await?;
    let admins = person::table
      .inner_join(person_aggregates::table)
      .select((Person::safe_columns_tuple(), person_aggregates::all_columns))
      .filter(person::admin.eq(true))
      .filter(person::deleted.eq(false))
      .order_by(person::published)
      .load::<PersonViewSafeTuple>(conn)
      .await?;

    Ok(Self::from_tuple_to_vec(admins))
  }

  pub async fn banned(pool: &DbPool) -> Result<Vec<Self>, Error> {
    let conn = &mut get_conn(pool).await?;
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
      .filter(person::deleted.eq(false))
      .load::<PersonViewSafeTuple>(conn)
      .await?;

    Ok(Self::from_tuple_to_vec(banned))
  }
}

#[derive(TypedBuilder)]
#[builder(field_defaults(default))]
pub struct PersonQuery<'a> {
  #[builder(!default)]
  pool: &'a DbPool,
  sort: Option<SortType>,
  search_term: Option<String>,
  page: Option<i64>,
  limit: Option<i64>,
}

impl<'a> PersonQuery<'a> {
  pub async fn list(self) -> Result<Vec<PersonViewSafe>, Error> {
    let conn = &mut get_conn(self.pool).await?;
    let mut query = person::table
      .inner_join(person_aggregates::table)
      .select((Person::safe_columns_tuple(), person_aggregates::all_columns))
      .into_boxed();

    if let Some(search_term) = self.search_term {
      let searcher = fuzzy_search(&search_term);
      query = query
        .filter(person::name.ilike(searcher.clone()))
        .or_filter(person::display_name.ilike(searcher));
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
      SortType::Old => query.order_by(person::published.asc()),
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

    let res = query.load::<PersonViewSafeTuple>(conn).await?;

    Ok(PersonViewSafe::from_tuple_to_vec(res))
  }
}

impl ViewToVec for PersonViewSafe {
  type DbTuple = PersonViewSafeTuple;
  fn from_tuple_to_vec(items: Vec<Self::DbTuple>) -> Vec<Self> {
    items
      .into_iter()
      .map(|a| Self {
        person: a.0,
        counts: a.1,
      })
      .collect::<Vec<Self>>()
  }
}
