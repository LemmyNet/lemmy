use crate::{
  newtypes::{CommunityId, DbUrl, InstanceId, PersonId},
  schema::{comment, community, instance, local_user, person, person_actions, post},
  source::person::{
    Person,
    PersonFollower,
    PersonFollowerForm,
    PersonInsertForm,
    PersonUpdateForm,
  },
  traits::{ApubActor, Crud, Followable},
  utils::{action_query, functions::lower, get_conn, naive_now, now, DbPool},
};
use chrono::{DateTime, Utc};
use diesel::{
  dsl::{self, insert_into},
  result::Error,
  CombineDsl,
  ExpressionMethods,
  JoinOnDsl,
  NullableExpressionMethods,
  QueryDsl,
};
use diesel_async::RunQueryDsl;

#[async_trait]
impl Crud for Person {
  type InsertForm = PersonInsertForm;
  type UpdateForm = PersonUpdateForm;
  type IdType = PersonId;
  async fn read(pool: &mut DbPool<'_>, person_id: PersonId) -> Result<Self, Error> {
    let conn = &mut get_conn(pool).await?;
    person::table
      .filter(person::deleted.eq(false))
      .find(person_id)
      .first::<Self>(conn)
      .await
  }

  async fn create(pool: &mut DbPool<'_>, form: &PersonInsertForm) -> Result<Self, Error> {
    let conn = &mut get_conn(pool).await?;
    insert_into(person::table)
      .values(form)
      .get_result::<Self>(conn)
      .await
  }
  async fn update(
    pool: &mut DbPool<'_>,
    person_id: PersonId,
    form: &PersonUpdateForm,
  ) -> Result<Self, Error> {
    let conn = &mut get_conn(pool).await?;
    diesel::update(person::table.find(person_id))
      .set(form)
      .get_result::<Self>(conn)
      .await
  }
}

impl Person {
  /// Update or insert the person.
  ///
  /// This is necessary for federation, because Activitypub doesnt distinguish between these actions.
  pub async fn upsert(pool: &mut DbPool<'_>, form: &PersonInsertForm) -> Result<Self, Error> {
    let conn = &mut get_conn(pool).await?;
    insert_into(person::table)
      .values(form)
      .on_conflict(person::actor_id)
      .do_update()
      .set(form)
      .get_result::<Self>(conn)
      .await
  }
  pub async fn delete_account(pool: &mut DbPool<'_>, person_id: PersonId) -> Result<Person, Error> {
    let conn = &mut get_conn(pool).await?;

    // Set the local user info to none
    diesel::update(local_user::table.filter(local_user::person_id.eq(person_id)))
      .set(local_user::email.eq::<Option<String>>(None))
      .execute(conn)
      .await?;

    diesel::update(person::table.find(person_id))
      .set((
        person::display_name.eq::<Option<String>>(None),
        person::avatar.eq::<Option<String>>(None),
        person::banner.eq::<Option<String>>(None),
        person::bio.eq::<Option<String>>(None),
        person::matrix_user_id.eq::<Option<String>>(None),
        person::deleted.eq(true),
        person::updated.eq(naive_now()),
      ))
      .get_result::<Self>(conn)
      .await
  }

  /// Lists local community ids for all posts and comments for a given creator.
  pub async fn list_local_community_ids(
    pool: &mut DbPool<'_>,
    for_creator_id: PersonId,
  ) -> Result<Vec<CommunityId>, Error> {
    let conn = &mut get_conn(pool).await?;
    comment::table
      .inner_join(post::table)
      .inner_join(community::table.on(post::community_id.eq(community::id)))
      .filter(community::local.eq(true))
      .filter(comment::creator_id.eq(for_creator_id))
      .select(community::id)
      .union(
        post::table
          .inner_join(community::table)
          .filter(community::local.eq(true))
          .filter(post::creator_id.eq(for_creator_id))
          .select(community::id),
      )
      .load::<CommunityId>(conn)
      .await
  }
}

impl PersonInsertForm {
  pub fn test_form(instance_id: InstanceId, name: &str) -> Self {
    Self::builder()
      .name(name.to_owned())
      .public_key("pubkey".to_string())
      .instance_id(instance_id)
      .build()
  }
}

#[async_trait]
impl ApubActor for Person {
  async fn read_from_apub_id(
    pool: &mut DbPool<'_>,
    object_id: &DbUrl,
  ) -> Result<Option<Self>, Error> {
    let conn = &mut get_conn(pool).await?;
    Ok(
      person::table
        .filter(person::deleted.eq(false))
        .filter(person::actor_id.eq(object_id))
        .first::<Person>(conn)
        .await
        .ok()
        .map(Into::into),
    )
  }

  async fn read_from_name(
    pool: &mut DbPool<'_>,
    from_name: &str,
    include_deleted: bool,
  ) -> Result<Person, Error> {
    let conn = &mut get_conn(pool).await?;
    let mut q = person::table
      .into_boxed()
      .filter(person::local.eq(true))
      .filter(lower(person::name).eq(from_name.to_lowercase()));
    if !include_deleted {
      q = q.filter(person::deleted.eq(false))
    }
    q.first::<Self>(conn).await
  }

  async fn read_from_name_and_domain(
    pool: &mut DbPool<'_>,
    person_name: &str,
    for_domain: &str,
  ) -> Result<Person, Error> {
    let conn = &mut get_conn(pool).await?;

    person::table
      .inner_join(instance::table)
      .filter(lower(person::name).eq(person_name.to_lowercase()))
      .filter(lower(instance::domain).eq(for_domain.to_lowercase()))
      .select(person::all_columns)
      .first::<Self>(conn)
      .await
  }
}

impl PersonFollower {
  fn as_select_unwrap() -> (
    person_actions::target_id,
    person_actions::person_id,
    dsl::AssumeNotNull<person_actions::followed>,
    dsl::AssumeNotNull<person_actions::follow_pending>,
  ) {
    (
      person_actions::target_id,
      person_actions::person_id,
      person_actions::followed.assume_not_null(),
      person_actions::follow_pending.assume_not_null(),
    )
  }
}

#[async_trait]
impl Followable for PersonFollower {
  type Form = PersonFollowerForm;
  async fn follow(pool: &mut DbPool<'_>, form: &PersonFollowerForm) -> Result<Self, Error> {
    let conn = &mut get_conn(pool).await?;
    let form = (form, person_actions::followed.eq(now().nullable()));
    insert_into(person_actions::table)
      .values(form)
      .on_conflict((person_actions::person_id, person_actions::target_id))
      .do_update()
      .set(form)
      .returning(Self::as_select_unwrap())
      .get_result::<Self>(conn)
      .await
  }
  async fn follow_accepted(_: &mut DbPool<'_>, _: CommunityId, _: PersonId) -> Result<Self, Error> {
    unimplemented!()
  }
  async fn unfollow(pool: &mut DbPool<'_>, form: &PersonFollowerForm) -> Result<usize, Error> {
    let conn = &mut get_conn(pool).await?;
    diesel::update(person_actions::table.find((form.follower_id, form.person_id)))
      .set((
        person_actions::followed.eq(None::<DateTime<Utc>>),
        person_actions::follow_pending.eq(None::<bool>),
      ))
      .execute(conn)
      .await
  }
}

impl PersonFollower {
  pub async fn list_followers(
    pool: &mut DbPool<'_>,
    for_person_id: PersonId,
  ) -> Result<Vec<Person>, Error> {
    let conn = &mut get_conn(pool).await?;
    action_query(person_actions::followed)
      .inner_join(person::table.on(person_actions::person_id.eq(person::id)))
      .filter(person_actions::target_id.eq(for_person_id))
      .select(person::all_columns)
      .load(conn)
      .await
  }
}

#[cfg(test)]
mod tests {
  #![allow(clippy::unwrap_used)]
  #![allow(clippy::indexing_slicing)]

  use crate::{
    source::{
      instance::Instance,
      person::{Person, PersonFollower, PersonFollowerForm, PersonInsertForm, PersonUpdateForm},
    },
    traits::{Crud, Followable},
    utils::build_db_pool_for_tests,
  };
  use pretty_assertions::assert_eq;
  use serial_test::serial;

  #[tokio::test]
  #[serial]
  async fn test_crud() {
    let pool = &build_db_pool_for_tests().await;
    let pool = &mut pool.into();

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

    let update_person_form = PersonUpdateForm {
      actor_id: Some(inserted_person.actor_id.clone()),
      ..Default::default()
    };
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
    let pool = &mut pool.into();
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
