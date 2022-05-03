use crate::structs::{LocalUserSettingsView, LocalUserView};
use diesel::{result::Error, *};
use lemmy_db_schema::{
  aggregates::structs::PersonAggregates,
  newtypes::{LocalUserId, PersonId},
  schema::{local_user, person, person_aggregates},
  source::{
    local_user::{LocalUser, LocalUserSettings},
    person::{Person, PersonSafe},
  },
  traits::{ToSafe, ToSafeSettings},
  utils::functions::lower,
};

type LocalUserViewTuple = (LocalUser, Person, PersonAggregates);

impl LocalUserView {
  pub fn read(conn: &PgConnection, local_user_id: LocalUserId) -> Result<Self, Error> {
    let (local_user, person, counts) = local_user::table
      .find(local_user_id)
      .inner_join(person::table)
      .inner_join(person_aggregates::table.on(person::id.eq(person_aggregates::person_id)))
      .select((
        local_user::all_columns,
        person::all_columns,
        person_aggregates::all_columns,
      ))
      .first::<LocalUserViewTuple>(conn)?;
    Ok(Self {
      local_user,
      person,
      counts,
    })
  }

  pub fn read_person(conn: &PgConnection, person_id: PersonId) -> Result<Self, Error> {
    let (local_user, person, counts) = local_user::table
      .filter(person::id.eq(person_id))
      .inner_join(person::table)
      .inner_join(person_aggregates::table.on(person::id.eq(person_aggregates::person_id)))
      .select((
        local_user::all_columns,
        person::all_columns,
        person_aggregates::all_columns,
      ))
      .first::<LocalUserViewTuple>(conn)?;
    Ok(Self {
      local_user,
      person,
      counts,
    })
  }

  // TODO check where this is used
  pub fn read_from_name(conn: &PgConnection, name: &str) -> Result<Self, Error> {
    let (local_user, person, counts) = local_user::table
      .filter(person::name.eq(name))
      .inner_join(person::table)
      .inner_join(person_aggregates::table.on(person::id.eq(person_aggregates::person_id)))
      .select((
        local_user::all_columns,
        person::all_columns,
        person_aggregates::all_columns,
      ))
      .first::<LocalUserViewTuple>(conn)?;
    Ok(Self {
      local_user,
      person,
      counts,
    })
  }

  pub fn find_by_email_or_name(conn: &PgConnection, name_or_email: &str) -> Result<Self, Error> {
    let (local_user, person, counts) = local_user::table
      .inner_join(person::table)
      .inner_join(person_aggregates::table.on(person::id.eq(person_aggregates::person_id)))
      .filter(
        lower(person::name)
          .eq(lower(name_or_email))
          .or(local_user::email.eq(name_or_email)),
      )
      .select((
        local_user::all_columns,
        person::all_columns,
        person_aggregates::all_columns,
      ))
      .first::<LocalUserViewTuple>(conn)?;
    Ok(Self {
      local_user,
      person,
      counts,
    })
  }

  pub fn find_by_email(conn: &PgConnection, from_email: &str) -> Result<Self, Error> {
    let (local_user, person, counts) = local_user::table
      .inner_join(person::table)
      .inner_join(person_aggregates::table.on(person::id.eq(person_aggregates::person_id)))
      .filter(local_user::email.eq(from_email))
      .select((
        local_user::all_columns,
        person::all_columns,
        person_aggregates::all_columns,
      ))
      .first::<LocalUserViewTuple>(conn)?;
    Ok(Self {
      local_user,
      person,
      counts,
    })
  }
}

type LocalUserSettingsViewTuple = (LocalUserSettings, PersonSafe, PersonAggregates);

impl LocalUserSettingsView {
  pub fn read(conn: &PgConnection, local_user_id: LocalUserId) -> Result<Self, Error> {
    let (local_user, person, counts) = local_user::table
      .find(local_user_id)
      .inner_join(person::table)
      .inner_join(person_aggregates::table.on(person::id.eq(person_aggregates::person_id)))
      .select((
        LocalUser::safe_settings_columns_tuple(),
        Person::safe_columns_tuple(),
        person_aggregates::all_columns,
      ))
      .first::<LocalUserSettingsViewTuple>(conn)?;
    Ok(Self {
      local_user,
      person,
      counts,
    })
  }
}
