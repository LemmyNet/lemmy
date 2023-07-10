use crate::structs::PersonView;
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
  schema,
  schema::{person, person_aggregates},
  source::person::Person,
  traits::JoinView,
  utils::{fuzzy_search, get_conn, limit_and_offset, DbPool},
  SortType,
};
use std::iter::Iterator;
use typed_builder::TypedBuilder;

type PersonViewTuple = (Person, PersonAggregates);

impl PersonView {
  pub async fn read(pool: &DbPool, person_id: PersonId) -> Result<Self, Error> {
    let conn = &mut get_conn(pool).await?;
    let res = person::table
      .find(person_id)
      .inner_join(person_aggregates::table)
      .select((person::all_columns, person_aggregates::all_columns))
      .first::<PersonViewTuple>(conn)
      .await?;
    Ok(Self::from_tuple(res))
  }

  pub async fn is_admin(pool: &DbPool, person_id: PersonId) -> Result<bool, Error> {
    use schema::person::dsl::{admin, id, person};
    let conn = &mut get_conn(pool).await?;
    let is_admin = person
      .filter(id.eq(person_id))
      .select(admin)
      .first::<bool>(conn)
      .await?;
    Ok(is_admin)
  }
  pub async fn admins(pool: &DbPool) -> Result<Vec<Self>, Error> {
    let conn = &mut get_conn(pool).await?;
    let admins = person::table
      .inner_join(person_aggregates::table)
      .select((person::all_columns, person_aggregates::all_columns))
      .filter(person::admin.eq(true))
      .filter(person::deleted.eq(false))
      .order_by(person::published)
      .load::<PersonViewTuple>(conn)
      .await?;

    Ok(admins.into_iter().map(Self::from_tuple).collect())
  }

  pub async fn banned(pool: &DbPool) -> Result<Vec<Self>, Error> {
    let conn = &mut get_conn(pool).await?;
    let banned = person::table
      .inner_join(person_aggregates::table)
      .select((person::all_columns, person_aggregates::all_columns))
      .filter(
        person::banned.eq(true).and(
          person::ban_expires
            .is_null()
            .or(person::ban_expires.gt(now)),
        ),
      )
      .filter(person::deleted.eq(false))
      .load::<PersonViewTuple>(conn)
      .await?;

    Ok(banned.into_iter().map(Self::from_tuple).collect())
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
  pub async fn list(self) -> Result<Vec<PersonView>, Error> {
    let conn = &mut get_conn(self.pool).await?;
    let mut query = person::table
      .inner_join(person_aggregates::table)
      .select((person::all_columns, person_aggregates::all_columns))
      .into_boxed();

    if let Some(search_term) = self.search_term {
      let searcher = fuzzy_search(&search_term);
      query = query
        .filter(person::name.ilike(searcher.clone()))
        .or_filter(person::display_name.ilike(searcher));
    }

    query = match self.sort.unwrap_or(SortType::Hot) {
      SortType::New | SortType::NewComments => query.order_by(person::published.desc()),
      SortType::Old => query.order_by(person::published.asc()),
      SortType::Hot | SortType::Active | SortType::TopAll => {
        query.order_by(person_aggregates::comment_score.desc())
      }
      SortType::MostComments => query.order_by(person_aggregates::comment_count.desc()),
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
      SortType::TopHour => query
        .filter(person::published.gt(now - 1.hours()))
        .order_by(person_aggregates::comment_score.desc()),
      SortType::TopSixHour => query
        .filter(person::published.gt(now - 6.hours()))
        .order_by(person_aggregates::comment_score.desc()),
      SortType::TopTwelveHour => query
        .filter(person::published.gt(now - 12.hours()))
        .order_by(person_aggregates::comment_score.desc()),
      SortType::TopThreeMonths => query
        .filter(person::published.gt(now - 3.months()))
        .order_by(person_aggregates::comment_score.desc()),
      SortType::TopSixMonths => query
        .filter(person::published.gt(now - 6.months()))
        .order_by(person_aggregates::comment_score.desc()),
      SortType::TopNineMonths => query
        .filter(person::published.gt(now - 9.months()))
        .order_by(person_aggregates::comment_score.desc()),
    };

    let (limit, offset) = limit_and_offset(self.page, self.limit)?;
    query = query.limit(limit).offset(offset);

    let res = query.load::<PersonViewTuple>(conn).await?;

    Ok(res.into_iter().map(PersonView::from_tuple).collect())
  }
}

impl JoinView for PersonView {
  type JoinTuple = PersonViewTuple;
  fn from_tuple(a: Self::JoinTuple) -> Self {
    Self {
      person: a.0,
      counts: a.1,
    }
  }
}
