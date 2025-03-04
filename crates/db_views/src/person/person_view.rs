use crate::structs::PersonView;
use diesel::{
  result::Error,
  BoolExpressionMethods,
  ExpressionMethods,
  NullableExpressionMethods,
  QueryDsl,
  SelectableHelper,
};
use diesel_async::RunQueryDsl;
use lemmy_db_schema::{
  newtypes::PersonId,
  schema::{local_user, person, person_aggregates},
  utils::{get_conn, now, DbPool},
};

impl PersonView {
  #[diesel::dsl::auto_type(no_type_alias)]
  fn joins() -> _ {
    person::table
      .inner_join(person_aggregates::table)
      .left_join(local_user::table)
  }

  pub async fn read(
    pool: &mut DbPool<'_>,
    person_id: PersonId,
    is_admin: bool,
  ) -> Result<Self, Error> {
    let conn = &mut get_conn(pool).await?;
    let mut query = Self::joins()
      .filter(person::id.eq(person_id))
      .select(Self::as_select())
      .into_boxed();

    if !is_admin {
      query = query.filter(person::deleted.eq(false))
    }

    query.first(conn).await
  }

  pub async fn admins(pool: &mut DbPool<'_>) -> Result<Vec<Self>, Error> {
    let conn = &mut get_conn(pool).await?;
    Self::joins()
      .filter(person::deleted.eq(false))
      .filter(local_user::admin.eq(true))
      .order_by(person::published)
      .select(Self::as_select())
      .load::<Self>(conn)
      .await
  }

  pub async fn banned(pool: &mut DbPool<'_>) -> Result<Vec<Self>, Error> {
    let conn = &mut get_conn(pool).await?;
    Self::joins()
      .filter(person::deleted.eq(false))
      .filter(
        person::banned.eq(true).and(
          person::ban_expires
            .is_null()
            .or(person::ban_expires.gt(now().nullable())),
        ),
      )
      .order_by(person::published)
      .select(Self::as_select())
      .load::<Self>(conn)
      .await
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
}
