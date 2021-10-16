use crate::{
  naive_now,
  newtypes::{DbUrl, PersonId},
  schema::person::dsl::*,
  source::person::{Person, PersonForm},
  traits::Crud,
};
use chrono::NaiveDateTime;
use diesel::{dsl::*, result::Error, ExpressionMethods, PgConnection, QueryDsl, RunQueryDsl, *};
use lemmy_apub_lib::traits::{ActorType, ApubObject};
use lemmy_utils::LemmyError;
use url::Url;

mod safe_type {
  use crate::{schema::person::columns::*, source::person::Person, traits::ToSafe};

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
      )
    }
  }
}

mod safe_type_alias_1 {
  use crate::{schema::person_alias_1::columns::*, source::person::PersonAlias1, traits::ToSafe};

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
  );

  impl ToSafe for PersonAlias1 {
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
      )
    }
  }
}

mod safe_type_alias_2 {
  use crate::{schema::person_alias_2::columns::*, source::person::PersonAlias2, traits::ToSafe};

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
  );

  impl ToSafe for PersonAlias2 {
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
      )
    }
  }
}

impl Crud for Person {
  type Form = PersonForm;
  type IdType = PersonId;
  fn read(conn: &PgConnection, person_id: PersonId) -> Result<Self, Error> {
    person
      .filter(deleted.eq(false))
      .find(person_id)
      .first::<Self>(conn)
  }
  fn delete(conn: &PgConnection, person_id: PersonId) -> Result<usize, Error> {
    diesel::delete(person.find(person_id)).execute(conn)
  }
  fn create(conn: &PgConnection, form: &PersonForm) -> Result<Self, Error> {
    insert_into(person).values(form).get_result::<Self>(conn)
  }
  fn update(conn: &PgConnection, person_id: PersonId, form: &PersonForm) -> Result<Self, Error> {
    diesel::update(person.find(person_id))
      .set(form)
      .get_result::<Self>(conn)
  }
}

impl Person {
  pub fn ban_person(conn: &PgConnection, person_id: PersonId, ban: bool) -> Result<Self, Error> {
    diesel::update(person.find(person_id))
      .set(banned.eq(ban))
      .get_result::<Self>(conn)
  }

  pub fn add_admin(conn: &PgConnection, person_id: PersonId, added: bool) -> Result<Self, Error> {
    diesel::update(person.find(person_id))
      .set(admin.eq(added))
      .get_result::<Self>(conn)
  }

  pub fn find_by_name(conn: &PgConnection, from_name: &str) -> Result<Person, Error> {
    person
      .filter(deleted.eq(false))
      .filter(local.eq(true))
      .filter(name.ilike(from_name))
      .first::<Person>(conn)
  }

  pub fn mark_as_updated(conn: &PgConnection, person_id: PersonId) -> Result<Person, Error> {
    diesel::update(person.find(person_id))
      .set((last_refreshed_at.eq(naive_now()),))
      .get_result::<Self>(conn)
  }

  pub fn delete_account(conn: &PgConnection, person_id: PersonId) -> Result<Person, Error> {
    use crate::schema::local_user;

    // Set the local user info to none
    diesel::update(local_user::table.filter(local_user::person_id.eq(person_id)))
      .set((
        local_user::email.eq::<Option<String>>(None),
        local_user::validator_time.eq(naive_now()),
      ))
      .execute(conn)?;

    diesel::update(person.find(person_id))
      .set((
        display_name.eq::<Option<String>>(None),
        bio.eq::<Option<String>>(None),
        matrix_user_id.eq::<Option<String>>(None),
        deleted.eq(true),
        updated.eq(naive_now()),
      ))
      .get_result::<Self>(conn)
  }

  pub fn upsert(conn: &PgConnection, person_form: &PersonForm) -> Result<Person, Error> {
    insert_into(person)
      .values(person_form)
      .on_conflict(actor_id)
      .do_update()
      .set(person_form)
      .get_result::<Self>(conn)
  }
}

impl ApubObject for Person {
  type DataType = PgConnection;

  fn last_refreshed_at(&self) -> Option<NaiveDateTime> {
    Some(self.last_refreshed_at)
  }

  fn read_from_apub_id(conn: &PgConnection, object_id: Url) -> Result<Option<Self>, LemmyError> {
    use crate::schema::person::dsl::*;
    let object_id: DbUrl = object_id.into();
    Ok(
      person
        .filter(deleted.eq(false))
        .filter(actor_id.eq(object_id))
        .first::<Self>(conn)
        .ok(),
    )
  }

  fn delete(self, conn: &PgConnection) -> Result<(), LemmyError> {
    use crate::schema::person::dsl::*;
    diesel::update(person.find(self.id))
      .set((deleted.eq(true), updated.eq(naive_now())))
      .get_result::<Self>(conn)?;
    Ok(())
  }
}

impl ActorType for Person {
  fn is_local(&self) -> bool {
    self.local
  }
  fn actor_id(&self) -> Url {
    self.actor_id.to_owned().into_inner()
  }
  fn name(&self) -> String {
    self.name.clone()
  }

  fn public_key(&self) -> Option<String> {
    self.public_key.to_owned()
  }

  fn private_key(&self) -> Option<String> {
    self.private_key.to_owned()
  }

  fn inbox_url(&self) -> Url {
    self.inbox_url.clone().into()
  }

  fn shared_inbox_url(&self) -> Option<Url> {
    self.shared_inbox_url.clone().map(|s| s.into_inner())
  }
}

#[cfg(test)]
mod tests {
  use crate::{establish_unpooled_connection, source::person::*, traits::Crud};

  #[test]
  fn test_crud() {
    let conn = establish_unpooled_connection();

    let new_person = PersonForm {
      name: "holly".into(),
      ..PersonForm::default()
    };

    let inserted_person = Person::create(&conn, &new_person).unwrap();

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
      actor_id: inserted_person.actor_id.to_owned(),
      bio: None,
      local: true,
      bot_account: false,
      admin: false,
      private_key: None,
      public_key: None,
      last_refreshed_at: inserted_person.published,
      inbox_url: inserted_person.inbox_url.to_owned(),
      shared_inbox_url: None,
      matrix_user_id: None,
    };

    let read_person = Person::read(&conn, inserted_person.id).unwrap();
    let updated_person = Person::update(&conn, inserted_person.id, &new_person).unwrap();
    let num_deleted = Person::delete(&conn, inserted_person.id).unwrap();

    assert_eq!(expected_person, read_person);
    assert_eq!(expected_person, inserted_person);
    assert_eq!(expected_person, updated_person);
    assert_eq!(1, num_deleted);
  }
}
