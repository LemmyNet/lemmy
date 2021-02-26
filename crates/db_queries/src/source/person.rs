use crate::{is_email_regex, ApubObject, Crud, ToSafeSettings};
use bcrypt::{hash, DEFAULT_COST};
use diesel::{dsl::*, result::Error, *};
use lemmy_db_schema::{
  naive_now,
  schema::person::dsl::*,
  source::person::{PersonForm, Person},
  Url,
};
use lemmy_utils::settings::Settings;

mod safe_type {
  use crate::ToSafe;
  use lemmy_db_schema::{schema::person::columns::*, source::person::Person};

  type Columns = (
  id,
  name,
  preferred_username,
  avatar,
  banned,
  published,
  updated,
  actor_id,
  bio,
  local,
  last_refreshed_at,
  banner,
  deleted,
  inbox_url,
  shared_inbox_url,
  );

  impl ToSafe for Person {
    type SafeColumns = Columns;
    fn safe_columns_tuple() -> Self::SafeColumns {
      (
        id,
        name,
        preferred_username,
        avatar,
        banned,
        published,
        updated,
        actor_id,
        bio,
        local,
        last_refreshed_at,
        banner,
        deleted,
        inbox_url,
        shared_inbox_url,
      )
    }
  }
}

mod safe_type_alias_1 {
  use crate::ToSafe;
  use lemmy_db_schema::{schema::person_alias_1::columns::*, source::person::PersonAlias1};

  type Columns = (
        id,
        name,
        preferred_username,
        avatar,
        banned,
        published,
        updated,
        actor_id,
        bio,
        local,
        last_refreshed_at,
        banner,
        deleted,
        inbox_url,
        shared_inbox_url,
  );

  impl ToSafe for PersonAlias1 {
    type SafeColumns = Columns;
    fn safe_columns_tuple() -> Self::SafeColumns {
      (
        id,
        name,
        preferred_username,
        avatar,
        banned,
        published,
        updated,
        actor_id,
        bio,
        local,
        last_refreshed_at,
        banner,
        deleted,
        inbox_url,
        shared_inbox_url,
      )
    }
  }
}

mod safe_type_alias_2 {
  use crate::ToSafe;
  use lemmy_db_schema::{schema::person_alias_2::columns::*, source::person::PersonAlias2};

  type Columns = (
        id,
        name,
        preferred_username,
        avatar,
        banned,
        published,
        updated,
        actor_id,
        bio,
        local,
        last_refreshed_at,
        banner,
        deleted,
        inbox_url,
        shared_inbox_url,
  );

  impl ToSafe for PersonAlias2 {
    type SafeColumns = Columns;
    fn safe_columns_tuple() -> Self::SafeColumns {
      (
        id,
        name,
        preferred_username,
        avatar,
        banned,
        published,
        updated,
        actor_id,
        bio,
        local,
        last_refreshed_at,
        banner,
        deleted,
        inbox_url,
        shared_inbox_url,
      )
    }
  }
}

impl Crud<PersonForm> for Person {
  fn read(conn: &PgConnection, person_id: i32) -> Result<Self, Error> {
    person
      .filter(deleted.eq(false))
      .find(person_id)
      .first::<Self>(conn)
  }
  fn delete(conn: &PgConnection, person_id: i32) -> Result<usize, Error> {
    diesel::delete(person.find(person_id)).execute(conn)
  }
  fn create(conn: &PgConnection, form: &PersonForm) -> Result<Self, Error> {
    insert_into(person).values(form).get_result::<Self>(conn)
  }
  fn update(conn: &PgConnection, person_id: i32, form: &PersonForm) -> Result<Self, Error> {
    diesel::update(person.find(person_id))
      .set(form)
      .get_result::<Self>(conn)
  }
}

impl ApubObject<PersonForm> for Person {
  fn read_from_apub_id(conn: &PgConnection, object_id: &Url) -> Result<Self, Error> {
    use lemmy_db_schema::schema::person::dsl::*;
    person
      .filter(deleted.eq(false))
      .filter(actor_id.eq(object_id))
      .first::<Self>(conn)
  }

  fn upsert(conn: &PgConnection, person_form: &PersonForm) -> Result<Person, Error> {
    insert_into(person)
      .values(person_form)
      .on_conflict(actor_id)
      .do_update()
      .set(person_form)
      .get_result::<Self>(conn)
  }
}

pub trait Person_ {
  fn register(conn: &PgConnection, form: &PersonForm) -> Result<Person, Error>;
  fn update_password(conn: &PgConnection, person_id: i32, new_password: &str)
    -> Result<Person, Error>;
  fn read_from_name(conn: &PgConnection, from_name: &str) -> Result<Person, Error>;
  fn add_admin(conn: &PgConnection, person_id: i32, added: bool) -> Result<Person, Error>;
  fn ban_person(conn: &PgConnection, person_id: i32, ban: bool) -> Result<Person, Error>;
  fn find_by_email_or_name(
    conn: &PgConnection,
    name_or_email: &str,
  ) -> Result<Person, Error>;
  fn find_by_name(conn: &PgConnection, name: &str) -> Result<Person, Error>;
  fn find_by_email(conn: &PgConnection, from_email: &str) -> Result<Person, Error>;
  fn get_profile_url(&self, hostname: &str) -> String;
  fn mark_as_updated(conn: &PgConnection, person_id: i32) -> Result<Person, Error>;
  fn delete_account(conn: &PgConnection, person_id: i32) -> Result<Person, Error>;
}

impl Person_ for Person {
  fn register(conn: &PgConnection, form: &PersonForm) -> Result<Self, Error> {
    let mut edited_person = form.clone();
    let password_hash =
      hash(&form.password_encrypted, DEFAULT_COST).expect("Couldn't hash password");
    edited_person.password_encrypted = password_hash;

    Self::create(&conn, &edited_person)
  }

  // TODO do more individual updates like these
  fn update_password(conn: &PgConnection, person_id: i32, new_password: &str) -> Result<Self, Error> {
    let password_hash = hash(new_password, DEFAULT_COST).expect("Couldn't hash password");

    diesel::update(person.find(person_id))
      .set((
        password_encrypted.eq(password_hash),
        updated.eq(naive_now()),
      ))
      .get_result::<Self>(conn)
  }

  fn read_from_name(conn: &PgConnection, from_name: &str) -> Result<Self, Error> {
    person
      .filter(local.eq(true))
      .filter(deleted.eq(false))
      .filter(name.eq(from_name))
      .first::<Self>(conn)
  }

  fn add_admin(conn: &PgConnection, person_id: i32, added: bool) -> Result<Self, Error> {
    diesel::update(person.find(person_id))
      .set(admin.eq(added))
      .get_result::<Self>(conn)
  }

  fn ban_person(conn: &PgConnection, person_id: i32, ban: bool) -> Result<Self, Error> {
    diesel::update(person.find(person_id))
      .set(banned.eq(ban))
      .get_result::<Self>(conn)
  }

  fn find_by_email_or_name(
    conn: &PgConnection,
    name_or_email: &str,
  ) -> Result<Self, Error> {
    if is_email_regex(name_or_email) {
      Self::find_by_email(conn, name_or_email)
    } else {
      Self::find_by_name(conn, name_or_email)
    }
  }

  fn find_by_name(conn: &PgConnection, name: &str) -> Result<Person, Error> {
    person
      .filter(deleted.eq(false))
      .filter(local.eq(true))
      .filter(name.ilike(name))
      .first::<Person>(conn)
  }

  fn find_by_email(conn: &PgConnection, from_email: &str) -> Result<Person, Error> {
    person
      .filter(deleted.eq(false))
      .filter(local.eq(true))
      .filter(email.eq(from_email))
      .first::<Person>(conn)
  }

  fn get_profile_url(&self, hostname: &str) -> String {
    format!(
      "{}://{}/u/{}",
      Settings::get().get_protocol_string(),
      hostname,
      self.name
    )
  }

  fn mark_as_updated(conn: &PgConnection, person_id: i32) -> Result<Person, Error> {
    diesel::update(person.find(person_id))
      .set((last_refreshed_at.eq(naive_now()),))
      .get_result::<Self>(conn)
  }

  fn delete_account(conn: &PgConnection, person_id: i32) -> Result<Person, Error> {
    diesel::update(person.find(person_id))
      .set((
        preferred_username.eq::<Option<String>>(None),
        email.eq::<Option<String>>(None),
        matrix_user_id.eq::<Option<String>>(None),
        bio.eq::<Option<String>>(None),
        deleted.eq(true),
        updated.eq(naive_now()),
      ))
      .get_result::<Self>(conn)
  }
}

#[cfg(test)]
mod tests {
  use crate::{establish_unpooled_connection, source::person::*, ListingType, SortType};

  #[test]
  fn test_crud() {
    let conn = establish_unpooled_connection();

    let new_person = PersonForm {
      name: "thommy".into(),
      preferred_username: None,
      avatar: None,
      banner: None,
      banned: Some(false),
      deleted: false,
      published: None,
      updated: None,
      actor_id: None,
      bio: None,
      local: true,
      private_key: None,
      public_key: None,
      last_refreshed_at: None,
      inbox_url: None,
      shared_inbox_url: None,
    };

    let inserted_person = Person::create(&conn, &new_person).unwrap();

    let expected_person = Person {
      id: inserted_person.id,
      name: "thommy".into(),
      preferred_username: None,
      avatar: None,
      banner: None,
      banned: false,
      deleted: false,
      published: inserted_person.published,
      updated: None,
      actor_id: inserted_person.actor_id.to_owned(),
      bio: None,
      local: true,
      private_key: None,
      public_key: None,
      last_refreshed_at: inserted_person.published,
      deleted: false,
      inbox_url: inserted_person.inbox_url.to_owned(),
      shared_inbox_url: None,
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
