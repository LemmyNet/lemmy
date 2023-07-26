use crate::structs::PersonView;
use diesel::{
  dsl::now,
  pg::Pg,
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
  utils::{fuzzy_search, get_conn, limit_and_offset, DbConn, DbPool, ListFn, Queries, ReadFn},
  PersonSortType,
};

type PersonViewTuple = (Person, PersonAggregates);

enum ListMode {
  Admins,
  Banned,
  Query(PersonQuery),
}

fn queries<'a>(
) -> Queries<impl ReadFn<'a, PersonView, PersonId>, impl ListFn<'a, PersonView, ListMode>> {
  let all_joins = |query: person::BoxedQuery<'a, Pg>| {
    query
      .inner_join(person_aggregates::table)
      .select((person::all_columns, person_aggregates::all_columns))
  };

  let read = move |mut conn: DbConn<'a>, person_id: PersonId| async move {
    all_joins(person::table.find(person_id).into_boxed())
      .first::<PersonViewTuple>(&mut conn)
      .await
  };

  let list = move |mut conn: DbConn<'a>, mode: ListMode| async move {
    let mut query = all_joins(person::table.into_boxed());
    match mode {
      ListMode::Admins => {
        query = query
          .filter(person::admin.eq(true))
          .filter(person::deleted.eq(false))
          .order_by(person::published);
      }
      ListMode::Banned => {
        query = query
          .filter(
            person::banned.eq(true).and(
              person::ban_expires
                .is_null()
                .or(person::ban_expires.gt(now)),
            ),
          )
          .filter(person::deleted.eq(false));
      }
      ListMode::Query(options) => {
        if let Some(search_term) = options.search_term {
          let searcher = fuzzy_search(&search_term);
          query = query
            .filter(person::name.ilike(searcher.clone()))
            .or_filter(person::display_name.ilike(searcher));
        }

        query = match options.sort.unwrap_or(PersonSortType::CommentScore) {
          PersonSortType::New => query.order_by(person::published.desc()),
          PersonSortType::Old => query.order_by(person::published.asc()),
          PersonSortType::MostComments => query.order_by(person_aggregates::comment_count.desc()),
          PersonSortType::CommentScore => query.order_by(person_aggregates::comment_score.desc()),
          PersonSortType::PostScore => query.order_by(person_aggregates::post_score.desc()),
          PersonSortType::PostCount => query.order_by(person_aggregates::post_count.desc()),
        };

        let (limit, offset) = limit_and_offset(options.page, options.limit)?;
        query = query.limit(limit).offset(offset);
      }
    }
    query.load::<PersonViewTuple>(&mut conn).await
  };

  Queries::new(read, list)
}

impl PersonView {
  pub async fn read(pool: &mut DbPool<'_>, person_id: PersonId) -> Result<Self, Error> {
    queries().read(pool, person_id).await
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
    queries().list(pool, ListMode::Admins).await
  }

  pub async fn banned(pool: &mut DbPool<'_>) -> Result<Vec<Self>, Error> {
    queries().list(pool, ListMode::Banned).await
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
    queries().list(pool, ListMode::Query(self)).await
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
