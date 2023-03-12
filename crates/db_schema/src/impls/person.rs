use crate::{
  newtypes::{CommunityId, DbUrl, PersonId},
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
  source::person::{
    Person,
    PersonFollower,
    PersonFollowerForm,
    PersonInsertForm,
    PersonUpdateForm,
  },
  traits::{ApubActor, Crud, Followable},
  utils::{functions::lower, get_conn, naive_now, DbPool},
};
use diesel::{dsl::insert_into, result::Error, ExpressionMethods, QueryDsl, TextExpressionMethods};
use diesel_async::RunQueryDsl;

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
      .filter(actor_id.like(format!("{protocol_domain}%")))
      .first::<Self>(conn)
      .await
  }
}

#[async_trait]
impl Followable for PersonFollower {
  type Form = PersonFollowerForm;
  async fn follow(pool: &DbPool, form: &PersonFollowerForm) -> Result<Self, Error> {
    use crate::schema::person_follower::dsl::{follower_id, person_follower, person_id};
    let conn = &mut get_conn(pool).await?;
    insert_into(person_follower)
      .values(form)
      .on_conflict((follower_id, person_id))
      .do_update()
      .set(form)
      .get_result::<Self>(conn)
      .await
  }
  async fn follow_accepted(_: &DbPool, _: CommunityId, _: PersonId) -> Result<Self, Error> {
    unimplemented!()
  }
  async fn unfollow(pool: &DbPool, form: &PersonFollowerForm) -> Result<usize, Error> {
    use crate::schema::person_follower::dsl::{follower_id, person_follower, person_id};
    let conn = &mut get_conn(pool).await?;
    diesel::delete(
      person_follower
        .filter(follower_id.eq(&form.follower_id))
        .filter(person_id.eq(&form.person_id)),
    )
    .execute(conn)
    .await
  }
}

impl PersonFollower {
  pub async fn list_followers(pool: &DbPool, person_id_: PersonId) -> Result<Vec<Person>, Error> {
    use crate::schema::{person, person_follower, person_follower::person_id};
    let conn = &mut get_conn(pool).await?;
    person_follower::table
      .inner_join(person::table)
      .filter(person_id.eq(person_id_))
      .select(person::all_columns)
      .load(conn)
      .await
  }
}

#[cfg(test)]
mod tests {
  use crate::{
    source::{
      instance::Instance,
      person::{Person, PersonFollower, PersonFollowerForm, PersonInsertForm, PersonUpdateForm},
    },
    traits::{Crud, Followable},
    utils::build_db_pool_for_tests,
  };
  use serial_test::serial;

  #[tokio::test]
  #[serial]
  async fn test_crud() {
    let pool = &build_db_pool_for_tests().await;

    let inserted_instance = Instance::read_or_create(pool, "my_domain.tld".to_string())
      .await
      .unwrap();

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

  #[tokio::test]
  #[serial]
  async fn follow() {
    let pool = &build_db_pool_for_tests().await;
    let inserted_instance = Instance::read_or_create(pool, "my_domain.tld".to_string())
      .await
      .unwrap();

    let person_form_1 = PersonInsertForm::builder()
      .name("erich".into())
      .public_key("pubkey".to_string())
      .instance_id(inserted_instance.id)
      .build();
    let person_1 = Person::create(pool, &person_form_1).await.unwrap();
    let person_form_2 = PersonInsertForm::builder()
      .name("michele".into())
      .public_key("pubkey".to_string())
      .instance_id(inserted_instance.id)
      .build();
    let person_2 = Person::create(pool, &person_form_2).await.unwrap();

    let follow_form = PersonFollowerForm {
      person_id: person_1.id,
      follower_id: person_2.id,
      pending: false,
    };
    let person_follower = PersonFollower::follow(pool, &follow_form).await.unwrap();
    assert_eq!(person_1.id, person_follower.person_id);
    assert_eq!(person_2.id, person_follower.follower_id);
    assert!(!person_follower.pending);

    let followers = PersonFollower::list_followers(pool, person_1.id)
      .await
      .unwrap();
    assert_eq!(vec![person_2], followers);

    let unfollow = PersonFollower::unfollow(pool, &follow_form).await.unwrap();
    assert_eq!(1, unfollow);
  }
}
