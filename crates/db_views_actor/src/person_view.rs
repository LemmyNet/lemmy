use crate::structs::PersonView;
use diesel::{
  dsl::now,
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
  PersonSortType,
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
  pub sort: Option<PersonSortType>,
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

    query = match self.sort.unwrap_or(PersonSortType::CommentScore) {
      PersonSortType::New => query.order_by(person::published.desc()),
      PersonSortType::Old => query.order_by(person::published.asc()),
      PersonSortType::MostComments => query.order_by(person_aggregates::comment_count.desc()),
      PersonSortType::CommentScore => query.order_by(person_aggregates::comment_score.desc()),
      PersonSortType::PostScore => query.order_by(person_aggregates::post_score.desc()),
      PersonSortType::PostCount => query.order_by(person_aggregates::post_count.desc()),
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
