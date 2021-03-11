use diesel::{result::Error, *};
use lemmy_db_queries::{aggregates::person_aggregates::PersonAggregates, ToSafe, ToSafeSettings};
use lemmy_db_schema::{
  schema::{local_user, person, person_aggregates},
  source::{
    local_user::{LocalUser, LocalUserSettings},
    person::{Person, PersonSafe},
  },
};
use serde::Serialize;

#[derive(Debug, Serialize, Clone)]
pub struct LocalUserView {
  pub person: Person,
  pub counts: PersonAggregates,
  pub local_user: LocalUser,
}

type LocalUserViewTuple = (Person, PersonAggregates, LocalUser);

impl LocalUserView {
  pub fn read_person(conn: &PgConnection, person_id: i32) -> Result<Self, Error> {
    let (person, counts, local_user) = person::table
      .find(person_id)
      .inner_join(person_aggregates::table)
      .inner_join(local_user::table)
      .select((
        person::all_columns,
        person_aggregates::all_columns,
        local_user::all_columns,
      ))
      .first::<LocalUserViewTuple>(conn)?;
    Ok(Self {
      person,
      counts,
      local_user,
    })
  }

  // TODO check where this is used
  pub fn read_from_name(conn: &PgConnection, name: &str) -> Result<Self, Error> {
    let (person, counts, local_user) = person::table
      .filter(person::name.eq(name))
      .inner_join(person_aggregates::table)
      .inner_join(local_user::table)
      .select((
        person::all_columns,
        person_aggregates::all_columns,
        local_user::all_columns,
      ))
      .first::<LocalUserViewTuple>(conn)?;
    Ok(Self {
      person,
      counts,
      local_user,
    })
  }

  pub fn find_by_email_or_name(conn: &PgConnection, name_or_email: &str) -> Result<Self, Error> {
    let (person, counts, local_user) = person::table
      .inner_join(person_aggregates::table)
      .inner_join(local_user::table)
      .filter(
        person::name
          .ilike(name_or_email)
          .or(local_user::email.ilike(name_or_email)),
      )
      .select((
        person::all_columns,
        person_aggregates::all_columns,
        local_user::all_columns,
      ))
      .first::<LocalUserViewTuple>(conn)?;
    Ok(Self {
      person,
      counts,
      local_user,
    })
  }

  pub fn find_by_email(conn: &PgConnection, from_email: &str) -> Result<Self, Error> {
    let (person, counts, local_user) = person::table
      .inner_join(person_aggregates::table)
      .inner_join(local_user::table)
      .filter(local_user::email.eq(from_email))
      .select((
        person::all_columns,
        person_aggregates::all_columns,
        local_user::all_columns,
      ))
      .first::<LocalUserViewTuple>(conn)?;
    Ok(Self {
      person,
      counts,
      local_user,
    })
  }
}

#[derive(Debug, Serialize, Clone)]
pub struct LocalUserSettingsView {
  pub person: PersonSafe,
  pub counts: PersonAggregates,
  pub local_user: LocalUserSettings,
}

type LocalUserSettingsViewTuple = (PersonSafe, PersonAggregates, LocalUserSettings);

impl LocalUserSettingsView {
  pub fn read(conn: &PgConnection, person_id: i32) -> Result<Self, Error> {
    let (person, counts, local_user) = person::table
      .find(person_id)
      .inner_join(person_aggregates::table)
      .inner_join(local_user::table)
      .select((
        Person::safe_columns_tuple(),
        person_aggregates::all_columns,
        LocalUser::safe_settings_columns_tuple(),
      ))
      .first::<LocalUserSettingsViewTuple>(conn)?;
    Ok(Self {
      person,
      counts,
      local_user,
    })
  }
}
