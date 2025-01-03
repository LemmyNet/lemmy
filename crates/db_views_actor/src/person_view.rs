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
  ListingType,
  PostSortType,
};
use serde::{Deserialize, Serialize};
use strum::{Display, EnumString};

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

fn post_to_person_sort_type(sort: PostSortType) -> PersonSortType {
  use PostSortType::*;
  match sort {
    Active | Hot | Controversial => PersonSortType::CommentScore,
    New | NewComments => PersonSortType::New,
    MostComments => PersonSortType::MostComments,
    Old => PersonSortType::Old,
    _ => PersonSortType::CommentScore,
  }
}

fn queries<'a>(
) -> Queries<impl ReadFn<'a, PersonView, (PersonId, bool)>, impl ListFn<'a, PersonView, ListMode>> {
  let all_joins = move |query: person::BoxedQuery<'a, Pg>| {
    query
      .inner_join(person_aggregates::table)
      .left_join(local_user::table)
      .select((
        person::all_columns,
        person_aggregates::all_columns,
        coalesce(local_user::admin.nullable(), false),
      ))
  };

  let read = move |mut conn: DbConn<'a>, params: (PersonId, bool)| async move {
    let (person_id, is_admin) = params;
    let mut query = all_joins(person::table.find(person_id).into_boxed());
    if !is_admin {
      query = query.filter(person::deleted.eq(false));
    }
    query.first(&mut conn).await
  };

  let list = move |mut conn: DbConn<'a>, mode: ListMode| async move {
    let mut query = all_joins(person::table.into_boxed()).filter(person::deleted.eq(false));
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
      ListMode::Query(o) => {
        if let Some(search_term) = o.search_term {
          let searcher = fuzzy_search(&search_term);
          query = query
            .filter(person::name.ilike(searcher.clone()))
            .or_filter(person::display_name.ilike(searcher));
        }

        let sort = o.sort.map(post_to_person_sort_type);
        query = match sort.unwrap_or(PersonSortType::CommentScore) {
          PersonSortType::New => query.order_by(person::published.desc()),
          PersonSortType::Old => query.order_by(person::published.asc()),
          PersonSortType::MostComments => query.order_by(person_aggregates::comment_count.desc()),
          PersonSortType::CommentScore => query.order_by(person_aggregates::comment_score.desc()),
          PersonSortType::PostScore => query.order_by(person_aggregates::post_score.desc()),
          PersonSortType::PostCount => query.order_by(person_aggregates::post_count.desc()),
        };

        let (limit, offset) = limit_and_offset(o.page, o.limit)?;
        query = query.limit(limit).offset(offset);

        if let Some(listing_type) = o.listing_type {
          query = match listing_type {
            // return nothing as its not possible to follow users
            ListingType::Subscribed => query.limit(0),
            ListingType::Local => query.filter(person::local.eq(true)),
            _ => query,
          };
        }
      }
    }
    query.load::<PersonView>(&mut conn).await
  };

  Queries::new(read, list)
}

impl PersonView {
  pub async fn read(
    pool: &mut DbPool<'_>,
    person_id: PersonId,
    is_admin: bool,
  ) -> Result<Self, Error> {
    queries().read(pool, (person_id, is_admin)).await
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
  pub sort: Option<PostSortType>,
  pub search_term: Option<String>,
  pub listing_type: Option<ListingType>,
  pub page: Option<i64>,
  pub limit: Option<i64>,
}

impl PersonQuery {
  pub async fn list(self, pool: &mut DbPool<'_>) -> Result<Vec<PersonView>, Error> {
    queries().list(pool, ListMode::Query(self)).await
  }
}

#[cfg(test)]
#[expect(clippy::indexing_slicing)]
mod tests {

  use super::*;
  use lemmy_db_schema::{
    assert_length,
    source::{
      instance::Instance,
      local_user::{LocalUser, LocalUserInsertForm, LocalUserUpdateForm},
      person::{Person, PersonInsertForm, PersonUpdateForm},
    },
    traits::Crud,
    utils::build_db_pool_for_tests,
  };
  use lemmy_utils::error::LemmyResult;
  use pretty_assertions::assert_eq;
  use serial_test::serial;

  struct Data {
    alice: Person,
    alice_local_user: LocalUser,
    bob: Person,
    bob_local_user: LocalUser,
  }

  async fn init_data(pool: &mut DbPool<'_>) -> LemmyResult<Data> {
    let inserted_instance = Instance::read_or_create(pool, "my_domain.tld".to_string()).await?;

    let alice_form = PersonInsertForm {
      local: Some(true),
      ..PersonInsertForm::test_form(inserted_instance.id, "alice")
    };
    let alice = Person::create(pool, &alice_form).await?;
    let alice_local_user_form = LocalUserInsertForm::test_form(alice.id);
    let alice_local_user = LocalUser::create(pool, &alice_local_user_form, vec![]).await?;

    let bob_form = PersonInsertForm {
      bot_account: Some(true),
      local: Some(false),
      ..PersonInsertForm::test_form(inserted_instance.id, "bob")
    };
    let bob = Person::create(pool, &bob_form).await?;
    let bob_local_user_form = LocalUserInsertForm::test_form(bob.id);
    let bob_local_user = LocalUser::create(pool, &bob_local_user_form, vec![]).await?;

    Ok(Data {
      alice,
      alice_local_user,
      bob,
      bob_local_user,
    })
  }

  async fn cleanup(data: Data, pool: &mut DbPool<'_>) -> LemmyResult<()> {
    LocalUser::delete(pool, data.alice_local_user.id).await?;
    LocalUser::delete(pool, data.bob_local_user.id).await?;
    Person::delete(pool, data.alice.id).await?;
    Person::delete(pool, data.bob.id).await?;
    Instance::delete(pool, data.bob.instance_id).await?;
    Ok(())
  }

  #[tokio::test]
  #[serial]
  async fn exclude_deleted() -> LemmyResult<()> {
    let pool = &build_db_pool_for_tests();
    let pool = &mut pool.into();
    let data = init_data(pool).await?;

    Person::update(
      pool,
      data.alice.id,
      &PersonUpdateForm {
        deleted: Some(true),
        ..Default::default()
      },
    )
    .await?;

    let read = PersonView::read(pool, data.alice.id, false).await;
    assert!(read.is_err());

    // only admin can view deleted users
    let read = PersonView::read(pool, data.alice.id, true).await;
    assert!(read.is_ok());

    let list = PersonQuery {
      sort: Some(PostSortType::New),
      ..Default::default()
    }
    .list(pool)
    .await?;
    assert_length!(1, list);
    assert_eq!(list[0].person.id, data.bob.id);

    cleanup(data, pool).await
  }

  #[tokio::test]
  #[serial]
  async fn list_banned() -> LemmyResult<()> {
    let pool = &build_db_pool_for_tests();
    let pool = &mut pool.into();
    let data = init_data(pool).await?;

    Person::update(
      pool,
      data.alice.id,
      &PersonUpdateForm {
        banned: Some(true),
        ..Default::default()
      },
    )
    .await?;

    let list = PersonView::banned(pool).await?;
    assert_length!(1, list);
    assert_eq!(list[0].person.id, data.alice.id);

    cleanup(data, pool).await
  }

  #[tokio::test]
  #[serial]
  async fn list_admins() -> LemmyResult<()> {
    let pool = &build_db_pool_for_tests();
    let pool = &mut pool.into();
    let data = init_data(pool).await?;

    LocalUser::update(
      pool,
      data.alice_local_user.id,
      &LocalUserUpdateForm {
        admin: Some(true),
        ..Default::default()
      },
    )
    .await?;

    let list = PersonView::admins(pool).await?;
    assert_length!(1, list);
    assert_eq!(list[0].person.id, data.alice.id);

    let is_admin = PersonView::read(pool, data.alice.id, false).await?.is_admin;
    assert!(is_admin);

    let is_admin = PersonView::read(pool, data.bob.id, false).await?.is_admin;
    assert!(!is_admin);

    cleanup(data, pool).await
  }

  #[tokio::test]
  #[serial]
  async fn listing_type() -> LemmyResult<()> {
    let pool = &build_db_pool_for_tests();
    let pool = &mut pool.into();
    let data = init_data(pool).await?;

    let list = PersonQuery {
      listing_type: Some(ListingType::Local),
      ..Default::default()
    }
    .list(pool)
    .await?;
    assert_length!(1, list);
    assert_eq!(list[0].person.id, data.alice.id);

    let list = PersonQuery {
      listing_type: Some(ListingType::All),
      ..Default::default()
    }
    .list(pool)
    .await?;
    assert_length!(2, list);

    cleanup(data, pool).await
  }
}
