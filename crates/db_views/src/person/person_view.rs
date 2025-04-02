use crate::{
  structs::{PersonView, SiteView},
  utils::{home_instance_person_join, local_instance_person_join},
};
use diesel::{
  BoolExpressionMethods,
  ExpressionMethods,
  NullableExpressionMethods,
  QueryDsl,
  SelectableHelper,
};
use diesel_async::RunQueryDsl;
use i_love_jesus::PaginatedQueryBuilder;
use lemmy_db_schema::{
  newtypes::{InstanceId, PaginationCursor, PersonId},
  schema::{instance_actions, local_user, person},
  source::person::{person_keys as key, Person},
  traits::PaginationCursorBuilder,
  utils::{get_conn, limit_fetch, now, DbPool},
};
use lemmy_utils::error::LemmyResult;

impl PaginationCursorBuilder for PersonView {
  type CursorData = Person;

  fn to_cursor(&self) -> PaginationCursor {
    PaginationCursor::new('P', self.person.id.0)
  }

  async fn from_cursor(
    cursor: &PaginationCursor,
    pool: &mut DbPool<'_>,
  ) -> LemmyResult<Self::CursorData> {
    let conn = &mut get_conn(pool).await?;
    let id = cursor.prefix_and_id()?.1;

    let token = person::table
      .select(Self::CursorData::as_select())
      .filter(person::id.eq(id))
      .first(conn)
      .await?;

    Ok(token)
  }
}

impl PersonView {
  #[diesel::dsl::auto_type(no_type_alias)]
  fn joins(local_instance_id: InstanceId) -> _ {
    let p: local_instance_person_join = local_instance_person_join(local_instance_id);
    person::table
      .left_join(p)
      .left_join(local_user::table)
      .left_join(home_instance_person_join())
  }

  pub async fn read(
    pool: &mut DbPool<'_>,
    person_id: PersonId,
    is_admin: bool,
  ) -> LemmyResult<Self> {
    let local_instance_id = SiteView::read_local(pool).await?.instance.id;
    let conn = &mut get_conn(pool).await?;
    let mut query = Self::joins(local_instance_id)
      .filter(person::id.eq(person_id))
      .select(Self::as_select())
      .into_boxed();

    if !is_admin {
      query = query.filter(person::deleted.eq(false))
    }

    Ok(query.first(conn).await?)
  }

  pub fn banned(&self) -> bool {
    self
      .instance_actions
      .as_ref()
      .is_some_and(|i| i.received_ban.is_some())
  }
}

#[derive(Default)]
pub struct PersonQuery {
  pub admins_only: Option<bool>,
  pub banned_only: Option<bool>,
  pub cursor_data: Option<Person>,
  pub page_back: Option<bool>,
  pub limit: Option<i64>,
}

impl PersonQuery {
  pub async fn list(
    self,
    local_instance_id: InstanceId,
    pool: &mut DbPool<'_>,
  ) -> LemmyResult<Vec<PersonView>> {
    let conn = &mut get_conn(pool).await?;
    let mut query = PersonView::joins(local_instance_id)
      .filter(person::deleted.eq(false))
      .select(PersonView::as_select())
      .into_boxed();

    // Filters
    if self.banned_only.unwrap_or_default() {
      query = query.filter(
        person::local
          .and(instance_actions::received_ban.is_not_null())
          .and(
            instance_actions::ban_expires
              .is_null()
              .or(instance_actions::ban_expires.gt(now().nullable())),
          ),
      );
    }

    if self.admins_only.unwrap_or_default() {
      query = query.filter(local_user::admin);
    } else {
      // Only use page limits if its not an admin fetch
      let limit = limit_fetch(self.limit)?;
      query = query.limit(limit);
    }

    let mut query = PaginatedQueryBuilder::new(query);

    if self.page_back.unwrap_or_default() {
      query = query.before(self.cursor_data).limit_and_offset_from_end();
    } else {
      query = query.after(self.cursor_data);
    }

    // Sorting by published
    query = query
      .then_desc(key::published)
      // Tie breaker
      .then_desc(key::id);

    let res = query.load::<PersonView>(conn).await?;
    Ok(res)
  }
}

#[cfg(test)]
#[expect(clippy::indexing_slicing)]
mod tests {

  use super::*;
  use crate::site::site_view::create_test_instance;
  use lemmy_db_schema::{
    assert_length,
    source::{
      instance::{Instance, InstanceActions, InstanceBanForm},
      local_user::{LocalUser, LocalUserInsertForm, LocalUserUpdateForm},
      person::{Person, PersonInsertForm, PersonUpdateForm},
    },
    traits::{Bannable, Crud},
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
    let instance = create_test_instance(pool).await?;

    let alice_form = PersonInsertForm {
      local: Some(true),
      ..PersonInsertForm::test_form(instance.id, "alice")
    };
    let alice = Person::create(pool, &alice_form).await?;
    let alice_local_user_form = LocalUserInsertForm::test_form(alice.id);
    let alice_local_user = LocalUser::create(pool, &alice_local_user_form, vec![]).await?;

    let bob_form = PersonInsertForm {
      bot_account: Some(true),
      local: Some(false),
      ..PersonInsertForm::test_form(instance.id, "bob")
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

    InstanceActions::ban(
      pool,
      &InstanceBanForm::new(data.alice.id, data.alice.instance_id, None),
    )
    .await?;

    let list = PersonQuery {
      banned_only: Some(true),
      ..Default::default()
    }
    .list(data.alice.instance_id, pool)
    .await?;
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

    let list = PersonQuery {
      admins_only: Some(true),
      ..Default::default()
    }
    .list(data.alice.instance_id, pool)
    .await?;
    assert_length!(1, list);
    assert_eq!(list[0].person.id, data.alice.id);

    let is_admin = PersonView::read(pool, data.alice.id, false).await?.is_admin;
    assert!(is_admin);

    let is_admin = PersonView::read(pool, data.bob.id, false).await?.is_admin;
    assert!(!is_admin);

    cleanup(data, pool).await
  }
}
