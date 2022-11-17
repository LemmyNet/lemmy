use crate::{
  newtypes::{DbUrl, PersonId},
  schema::person::dsl::{
    actor_id,
    avatar,
    banner,
    bio,
    deleted,
    display_name,
    local,
    matrix_user_id,
    name,
    person,
    updated,
  },
  source::person::{Person, PersonInsertForm, PersonUpdateForm},
  traits::{ApubActor, Crud},
  utils::{functions::lower, get_conn, naive_now, DbPool},
};
use diesel::{dsl::insert_into, result::Error, ExpressionMethods, QueryDsl, TextExpressionMethods};
use diesel_async::RunQueryDsl;

mod safe_type {
  use crate::{
    schema::person::columns::{
      actor_id,
      admin,
      avatar,
      ban_expires,
      banned,
      banner,
      bio,
      bot_account,
      deleted,
      display_name,
      id,
      inbox_url,
      instance_id,
      local,
      matrix_user_id,
      name,
      published,
      shared_inbox_url,
      updated,
    },
    source::person::Person,
    traits::ToSafe,
  };

  type Columns = (
    id,
    name,
    display_name,
    avatar,
    banned,
    published,
    updated,
    actor_id,
    bio,
    local,
    banner,
    deleted,
    inbox_url,
    shared_inbox_url,
    matrix_user_id,
    admin,
    bot_account,
    ban_expires,
    instance_id,
  );

  impl ToSafe for Person {
    type SafeColumns = Columns;
    fn safe_columns_tuple() -> Self::SafeColumns {
      (
        id,
        name,
        display_name,
        avatar,
        banned,
        published,
        updated,
        actor_id,
        bio,
        local,
        banner,
        deleted,
        inbox_url,
        shared_inbox_url,
        matrix_user_id,
        admin,
        bot_account,
        ban_expires,
        instance_id,
      )
    }
  }
}

#[async_trait]
impl Crud for Person {
  type InsertForm = PersonInsertForm;
  type UpdateForm = PersonUpdateForm;
  type IdType = PersonId;
  async fn read(pool: &DbPool, person_id: PersonId) -> Result<Self, Error> {
    let conn = &mut get_conn(pool).await?;
    person
      .filter(deleted.eq(false))
      .find(person_id)
      .first::<Self>(conn)
      .await
  }
  async fn delete(pool: &DbPool, person_id: PersonId) -> Result<usize, Error> {
    let conn = &mut get_conn(pool).await?;
    diesel::delete(person.find(person_id)).execute(conn).await
  }
  async fn create(pool: &DbPool, form: &PersonInsertForm) -> Result<Self, Error> {
    let conn = &mut get_conn(pool).await?;
    insert_into(person)
      .values(form)
      .on_conflict(actor_id)
      .do_update()
      .set(form)
      .get_result::<Self>(conn)
      .await
  }
  async fn update(
    pool: &DbPool,
    person_id: PersonId,
    form: &PersonUpdateForm,
  ) -> Result<Self, Error> {
    let conn = &mut get_conn(pool).await?;
    diesel::update(person.find(person_id))
      .set(form)
      .get_result::<Self>(conn)
      .await
  }
}

impl Person {
  pub async fn delete_account(pool: &DbPool, person_id: PersonId) -> Result<Person, Error> {
    use crate::schema::local_user;
    let conn = &mut get_conn(pool).await?;

    // Set the local user info to none
    diesel::update(local_user::table.filter(local_user::person_id.eq(person_id)))
      .set((
        local_user::email.eq::<Option<String>>(None),
        local_user::validator_time.eq(naive_now()),
      ))
      .execute(conn)
      .await?;

    diesel::update(person.find(person_id))
      .set((
        display_name.eq::<Option<String>>(None),
        avatar.eq::<Option<String>>(None),
        banner.eq::<Option<String>>(None),
        bio.eq::<Option<String>>(None),
        matrix_user_id.eq::<Option<String>>(None),
        deleted.eq(true),
        updated.eq(naive_now()),
      ))
      .get_result::<Self>(conn)
      .await
  }
}

pub fn is_banned(banned_: bool, expires: Option<chrono::NaiveDateTime>) -> bool {
  if let Some(expires) = expires {
    banned_ && expires.gt(&naive_now())
  } else {
    banned_
  }
}

#[async_trait]
impl ApubActor for Person {
  async fn read_from_apub_id(pool: &DbPool, object_id: &DbUrl) -> Result<Option<Self>, Error> {
    let conn = &mut get_conn(pool).await?;
    Ok(
      person
        .filter(deleted.eq(false))
        .filter(actor_id.eq(object_id))
        .first::<Person>(conn)
        .await
        .ok()
        .map(Into::into),
    )
  }

  async fn read_from_name(
    pool: &DbPool,
    from_name: &str,
    include_deleted: bool,
  ) -> Result<Person, Error> {
    let conn = &mut get_conn(pool).await?;
    let mut q = person
      .into_boxed()
      .filter(local.eq(true))
      .filter(lower(name).eq(lower(from_name)));
    if !include_deleted {
      q = q.filter(deleted.eq(false))
    }
    q.first::<Self>(conn).await
  }

  async fn read_from_name_and_domain(
    pool: &DbPool,
    person_name: &str,
    protocol_domain: &str,
  ) -> Result<Person, Error> {
    let conn = &mut get_conn(pool).await?;
    person
      .filter(lower(name).eq(lower(person_name)))
      .filter(actor_id.like(format!("{}%", protocol_domain)))
      .first::<Self>(conn)
      .await
  }
}

#[cfg(test)]
mod tests {
  use crate::{
    source::{
      instance::Instance,
      person::{Person, PersonInsertForm, PersonUpdateForm},
    },
    traits::Crud,
    utils::build_db_pool_for_tests,
  };
  use serial_test::serial;

  #[tokio::test]
  #[serial]
  async fn test_crud() {
    let pool = &build_db_pool_for_tests().await;

    let inserted_instance = Instance::create(pool, "my_domain.tld").await.unwrap();

    let new_person = PersonInsertForm::builder()
      .name("holly".into())
      .public_key("nada".to_owned())
      .instance_id(inserted_instance.id)
      .build();

    let inserted_person = Person::create(pool, &new_person).await.unwrap();

    let expected_person = Person {
      id: inserted_person.id,
      name: "holly".into(),
      display_name: None,
      avatar: None,
      banner: None,
      banned: false,
      deleted: false,
      published: inserted_person.published,
      updated: None,
      actor_id: inserted_person.actor_id.clone(),
      bio: None,
      local: true,
      bot_account: false,
      admin: false,
      private_key: None,
      public_key: "nada".to_owned(),
      last_refreshed_at: inserted_person.published,
      inbox_url: inserted_person.inbox_url.clone(),
      shared_inbox_url: None,
      matrix_user_id: None,
      ban_expires: None,
      instance_id: inserted_instance.id,
    };

    let read_person = Person::read(pool, inserted_person.id).await.unwrap();

    let update_person_form = PersonUpdateForm::builder()
      .actor_id(Some(inserted_person.actor_id.clone()))
      .build();
    let updated_person = Person::update(pool, inserted_person.id, &update_person_form)
      .await
      .unwrap();

    let num_deleted = Person::delete(pool, inserted_person.id).await.unwrap();
    Instance::delete(pool, inserted_instance.id).await.unwrap();

    assert_eq!(expected_person, read_person);
    assert_eq!(expected_person, inserted_person);
    assert_eq!(expected_person, updated_person);
    assert_eq!(1, num_deleted);
  }
}
