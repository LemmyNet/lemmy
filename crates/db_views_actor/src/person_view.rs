use crate::structs::PersonView;
use diesel::{
  dsl::{now, sql, IntervalDsl},
  result::Error,
  sql_types,
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

type PersonViewTuple = (Person, PersonAggregates);

impl PersonView {
  pub async fn read(pool: &mut DbPool<'_>, person_id: PersonId) -> Result<Self, Error> {
    let conn = &mut get_conn(pool).await?;
    let res = person::table
      .find(person_id)
      .inner_join(person_aggregates::table)
      .select((person::all_columns, person_aggregates::all_columns))
      .first::<PersonViewTuple>(conn)
      .await?;
    Ok(Self::from_tuple(res))
  }

  pub async fn is_admin(pool: &mut DbPool<'_>, person_id: PersonId) -> Result<bool, Error> {
    use schema::person::dsl::{admin, id, person};
    let conn = &mut get_conn(pool).await?;
    let is_admin = person
      .filter(id.eq(person_id))
      .select(admin)
      .first::<bool>(conn)
      .await?;
    Ok(is_admin)
  }
  pub async fn admins(pool: &mut DbPool<'_>) -> Result<Vec<Self>, Error> {
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

  pub async fn banned(pool: &mut DbPool<'_>) -> Result<Vec<Self>, Error> {
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

#[derive(Default)]
pub struct PersonQuery {
  pub sort: Option<SortType>,
  pub search_term: Option<String>,
  pub page: Option<i64>,
  pub limit: Option<i64>,
}

impl PersonQuery {
  pub async fn list(self, pool: &mut DbPool<'_>) -> Result<Vec<PersonView>, Error> {
    let conn = &mut get_conn(pool).await?;
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

    // Time range filters
    query = match self.sort.unwrap_or(SortType::Hot) {
      SortType::TopYear | SortType::BestYear => query.filter(person::published.gt(now - 1.years())),
      SortType::TopMonth | SortType::BestMonth => {
        query.filter(person::published.gt(now - 1.months()))
      }
      SortType::TopWeek | SortType::BestWeek => query.filter(person::published.gt(now - 1.weeks())),
      SortType::TopDay | SortType::BestDay => query.filter(person::published.gt(now - 1.days())),
      SortType::TopSixHour | SortType::BestSixHour => {
        query.filter(person::published.gt(now - 6.hours()))
      }
      SortType::TopThreeMonths | SortType::BestThreeMonths => {
        query.filter(person::published.gt(now - 3.months()))
      }
      SortType::TopSixMonths | SortType::BestSixMonths => {
        query.filter(person::published.gt(now - 6.months()))
      }
      SortType::TopNineMonths | SortType::BestNineMonths => {
        query.filter(person::published.gt(now - 9.months()))
      }
      SortType::TopTwelveHour | SortType::BestTwelveHour => {
        query.filter(person::published.gt(now - 12.hours()))
      }
      SortType::TopHour | SortType::BestHour => query.filter(person::published.gt(now - 1.hours())),

      _ => query,
    };

    query = match self.sort.unwrap_or(SortType::Hot) {
            SortType::New | SortType::NewComments => query.order_by(person::published.desc()),
            SortType::Old => query.order_by(person::published.asc()),
            SortType::MostComments => query.order_by(person_aggregates::comment_count.desc()),

          SortType::Hot |
          SortType::Active |
          SortType::TopAll |
          SortType::TopYear |
          SortType::TopMonth |
          SortType::TopWeek |
          SortType::TopDay |
          SortType::TopHour |
          SortType::TopSixHour |
          SortType::TopTwelveHour |
          SortType::TopThreeMonths |
          SortType::TopSixMonths |
          SortType::TopNineMonths => {
            query.order_by(person_aggregates::comment_score.desc())
            .then_order_by(person::published.desc())
          },
          SortType::BestAll |
          SortType::BestYear |
          SortType::BestThreeMonths |
          SortType::BestSixMonths |
          SortType::BestNineMonths |
          SortType::BestMonth |
          SortType::BestWeek |
          SortType::BestDay |
          SortType::BestTwelveHour |
          SortType::BestSixHour |
          SortType::BestHour=> {
            query
            .then_order_by(
                sql::<sql_types::BigInt>(
                "row_number() OVER (PARTITION BY \"person\".\"instance_id\" ORDER BY \"person_aggregates\".\"comment_score\" DESC)"
            ).asc()
            )
            .then_order_by(person::published.desc())
          },
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
