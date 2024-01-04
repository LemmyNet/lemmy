use crate::structs::PersonView;
use diesel::{
  pg::Pg,
  result::Error,
  BoolExpressionMethods,
  ExpressionMethods,
  NullableExpressionMethods,
  PgTextExpressionMethods,
  QueryDsl,
};
use diesel_async::RunQueryDsl;
use lemmy_db_schema::{
  newtypes::PersonId,
  schema::{local_user, person, person_aggregates},
  utils::{
    functions::coalesce,
    fuzzy_search,
    limit_and_offset,
    now,
    DbConn,
    DbPool,
    ListFn,
    Queries,
    ReadFn,
  },
  SortType,
};
use serde::{Deserialize, Serialize};
use strum_macros::{Display, EnumString};

enum ListMode {
  Admins,
  Banned,
  Query(PersonQuery),
}

#[derive(EnumString, Display, Debug, Serialize, Deserialize, Clone, Copy)]
/// The person sort types. Converted automatically from `SortType`
enum PersonSortType {
  New,
  Old,
  MostComments,
  CommentScore,
  PostScore,
  PostCount,
}

fn post_to_person_sort_type(sort: SortType) -> PersonSortType {
  match sort {
    SortType::Active | SortType::Hot | SortType::Controversial => PersonSortType::CommentScore,
    SortType::New | SortType::NewComments => PersonSortType::New,
    SortType::MostComments => PersonSortType::MostComments,
    SortType::Old => PersonSortType::Old,
    _ => PersonSortType::CommentScore,
  }
}

fn queries<'a>(
) -> Queries<impl ReadFn<'a, PersonView, PersonId>, impl ListFn<'a, PersonView, ListMode>> {
  let all_joins = move |query: person::BoxedQuery<'a, Pg>| {
    query
      .inner_join(person_aggregates::table)
      .left_join(local_user::table)
      .filter(person::deleted.eq(false))
      .select((
        person::all_columns,
        person_aggregates::all_columns,
        coalesce(local_user::admin.nullable(), false),
      ))
  };

  let read = move |mut conn: DbConn<'a>, person_id: PersonId| async move {
    all_joins(person::table.find(person_id).into_boxed())
      .first::<PersonView>(&mut conn)
      .await
  };

  let list = move |mut conn: DbConn<'a>, mode: ListMode| async move {
    let mut query = all_joins(person::table.into_boxed());
    match mode {
      ListMode::Admins => {
        query = query
          .filter(local_user::admin.eq(true))
          .filter(person::deleted.eq(false))
          .order_by(person::published);
      }
      ListMode::Banned => {
        query = query
          .filter(
            person::banned.eq(true).and(
              person::ban_expires
                .is_null()
                .or(person::ban_expires.gt(now().nullable())),
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

        let sort = options.sort.map(post_to_person_sort_type);
        query = match sort.unwrap_or(PersonSortType::CommentScore) {
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
    query.load::<PersonView>(&mut conn).await
  };

  Queries::new(read, list)
}

impl PersonView {
  pub async fn read(pool: &mut DbPool<'_>, person_id: PersonId) -> Result<Self, Error> {
    queries().read(pool, person_id).await
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
  pub sort: Option<SortType>,
  pub search_term: Option<String>,
  pub page: Option<i64>,
  pub limit: Option<i64>,
}

impl PersonQuery {
  pub async fn list(self, pool: &mut DbPool<'_>) -> Result<Vec<PersonView>, Error> {
    queries().list(pool, ListMode::Query(self)).await
  }
}

#[cfg(test)]
mod tests {
  #![allow(clippy::unwrap_used)]
  #![allow(clippy::indexing_slicing)]

  use super::*;
  use diesel::NotFound;
  use lemmy_db_schema::{
    source::{
      instance::Instance,
      local_user::{LocalUser, LocalUserInsertForm, LocalUserUpdateForm},
      person::{Person, PersonInsertForm, PersonUpdateForm},
    },
    traits::Crud,
    utils::build_db_pool_for_tests,
  };
  use pretty_assertions::assert_eq;
  use serial_test::serial;

  struct Data {
    alice: Person,
    alice_local_user: LocalUser,
    bob: Person,
    bob_local_user: LocalUser,
  }

  async fn init_data(pool: &mut DbPool<'_>) -> Data {
    let inserted_instance = Instance::read_or_create(pool, "my_domain.tld".to_string())
      .await
      .unwrap();

    let alice_form = PersonInsertForm::builder()
      .name("alice".to_string())
      .public_key("pubkey".to_string())
      .instance_id(inserted_instance.id)
      .build();
    let alice = Person::create(pool, &alice_form).await.unwrap();
    let alice_local_user_form = LocalUserInsertForm::builder()
      .person_id(alice.id)
      .password_encrypted(String::new())
      .build();
    let alice_local_user = LocalUser::create(pool, &alice_local_user_form)
      .await
      .unwrap();

    let bob_form = PersonInsertForm::builder()
      .name("bob".to_string())
      .bot_account(Some(true))
      .public_key("pubkey".to_string())
      .instance_id(inserted_instance.id)
      .build();
    let bob = Person::create(pool, &bob_form).await.unwrap();
    let bob_local_user_form = LocalUserInsertForm::builder()
      .person_id(bob.id)
      .password_encrypted(String::new())
      .build();
    let bob_local_user = LocalUser::create(pool, &bob_local_user_form).await.unwrap();

    Data {
      alice,
      alice_local_user,
      bob,
      bob_local_user,
    }
  }

  async fn cleanup(data: Data, pool: &mut DbPool<'_>) {
    LocalUser::delete(pool, data.alice_local_user.id)
      .await
      .unwrap();
    LocalUser::delete(pool, data.bob_local_user.id)
      .await
      .unwrap();
    Person::delete(pool, data.alice.id).await.unwrap();
    Person::delete(pool, data.bob.id).await.unwrap();
    Instance::delete(pool, data.bob.instance_id).await.unwrap();
  }

  #[tokio::test]
  #[serial]
  async fn exclude_deleted() {
    let pool = &build_db_pool_for_tests().await;
    let pool = &mut pool.into();
    let data = init_data(pool).await;

    Person::update(
      pool,
      data.alice.id,
      &PersonUpdateForm {
        deleted: Some(true),
        ..Default::default()
      },
    )
    .await
    .unwrap();

    let read = PersonView::read(pool, data.alice.id).await;
    assert_eq!(read.err(), Some(NotFound));

    let list = PersonQuery {
      sort: Some(SortType::New),
      ..Default::default()
    }
    .list(pool)
    .await
    .unwrap();
    assert_eq!(list.len(), 1);
    assert_eq!(list[0].person.id, data.bob.id);

    cleanup(data, pool).await;
  }

  #[tokio::test]
  #[serial]
  async fn list_banned() {
    let pool = &build_db_pool_for_tests().await;
    let pool = &mut pool.into();
    let data = init_data(pool).await;

    Person::update(
      pool,
      data.alice.id,
      &PersonUpdateForm {
        banned: Some(true),
        ..Default::default()
      },
    )
    .await
    .unwrap();

    let list = PersonView::banned(pool).await.unwrap();
    assert_eq!(list.len(), 1);
    assert_eq!(list[0].person.id, data.alice.id);

    cleanup(data, pool).await;
  }

  #[tokio::test]
  #[serial]
  async fn list_admins() {
    let pool = &build_db_pool_for_tests().await;
    let pool = &mut pool.into();
    let data = init_data(pool).await;

    LocalUser::update(
      pool,
      data.alice_local_user.id,
      &LocalUserUpdateForm {
        admin: Some(true),
        ..Default::default()
      },
    )
    .await
    .unwrap();

    let list = PersonView::admins(pool).await.unwrap();
    assert_eq!(list.len(), 1);
    assert_eq!(list[0].person.id, data.alice.id);

    let is_admin = PersonView::read(pool, data.alice.id)
      .await
      .unwrap()
      .is_admin;
    assert!(is_admin);

    let is_admin = PersonView::read(pool, data.bob.id).await.unwrap().is_admin;
    assert!(!is_admin);

    cleanup(data, pool).await;
  }
}
