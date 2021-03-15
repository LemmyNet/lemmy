use crate::Crud;
use bcrypt::{hash, DEFAULT_COST};
use diesel::{dsl::*, result::Error, *};
use lemmy_db_schema::{
  schema::local_user::dsl::*,
  source::local_user::{LocalUser, LocalUserForm},
};

mod safe_type {
  use crate::ToSafe;
  use lemmy_db_schema::{schema::local_user::columns::*, source::local_user::LocalUser};

  type Columns = (id, person_id, admin, matrix_user_id);

  impl ToSafe for LocalUser {
    type SafeColumns = Columns;
    fn safe_columns_tuple() -> Self::SafeColumns {
      (id, person_id, admin, matrix_user_id)
    }
  }
}

mod safe_settings_type {
  use crate::ToSafeSettings;
  use lemmy_db_schema::{schema::local_user::columns::*, source::local_user::LocalUser};

  type Columns = (
    id,
    person_id,
    email,
    admin,
    show_nsfw,
    theme,
    default_sort_type,
    default_listing_type,
    lang,
    show_avatars,
    send_notifications_to_email,
    matrix_user_id,
  );

  impl ToSafeSettings for LocalUser {
    type SafeSettingsColumns = Columns;

    /// Includes everything but the hashed password
    fn safe_settings_columns_tuple() -> Self::SafeSettingsColumns {
      (
        id,
        person_id,
        email,
        admin,
        show_nsfw,
        theme,
        default_sort_type,
        default_listing_type,
        lang,
        show_avatars,
        send_notifications_to_email,
        matrix_user_id,
      )
    }
  }
}

pub trait LocalUser_ {
  fn register(conn: &PgConnection, form: &LocalUserForm) -> Result<LocalUser, Error>;
  fn update_password(
    conn: &PgConnection,
    local_user_id: i32,
    new_password: &str,
  ) -> Result<LocalUser, Error>;
  fn add_admin(conn: &PgConnection, person_id: i32, added: bool) -> Result<LocalUser, Error>;
}

impl LocalUser_ for LocalUser {
  fn register(conn: &PgConnection, form: &LocalUserForm) -> Result<Self, Error> {
    let mut edited_user = form.clone();
    let password_hash =
      hash(&form.password_encrypted, DEFAULT_COST).expect("Couldn't hash password");
    edited_user.password_encrypted = password_hash;

    Self::create(&conn, &edited_user)
  }

  fn update_password(
    conn: &PgConnection,
    local_user_id: i32,
    new_password: &str,
  ) -> Result<Self, Error> {
    let password_hash = hash(new_password, DEFAULT_COST).expect("Couldn't hash password");

    diesel::update(local_user.find(local_user_id))
      .set((password_encrypted.eq(password_hash),))
      .get_result::<Self>(conn)
  }

  fn add_admin(conn: &PgConnection, for_person_id: i32, added: bool) -> Result<Self, Error> {
    diesel::update(local_user.filter(person_id.eq(for_person_id)))
      .set(admin.eq(added))
      .get_result::<Self>(conn)
  }
}

impl Crud<LocalUserForm> for LocalUser {
  fn read(conn: &PgConnection, local_user_id: i32) -> Result<Self, Error> {
    local_user.find(local_user_id).first::<Self>(conn)
  }
  fn delete(conn: &PgConnection, local_user_id: i32) -> Result<usize, Error> {
    diesel::delete(local_user.find(local_user_id)).execute(conn)
  }
  fn create(conn: &PgConnection, form: &LocalUserForm) -> Result<Self, Error> {
    insert_into(local_user)
      .values(form)
      .get_result::<Self>(conn)
  }
  fn update(conn: &PgConnection, local_user_id: i32, form: &LocalUserForm) -> Result<Self, Error> {
    diesel::update(local_user.find(local_user_id))
      .set(form)
      .get_result::<Self>(conn)
  }
}
