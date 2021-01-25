use crate::{is_email_regex, ApubObject, Crud, ToSafeSettings};
use bcrypt::{hash, DEFAULT_COST};
use diesel::{dsl::*, result::Error, *};
use lemmy_db_schema::{
  naive_now,
  schema::user_::dsl::*,
  source::user::{UserForm, UserSafeSettings, User_},
  Url,
};
use lemmy_utils::settings::Settings;

mod safe_type {
  use crate::ToSafe;
  use lemmy_db_schema::{schema::user_::columns::*, source::user::User_};

  type Columns = (
    id,
    name,
    preferred_username,
    avatar,
    admin,
    banned,
    published,
    updated,
    matrix_user_id,
    actor_id,
    bio,
    local,
    banner,
    deleted,
  );

  impl ToSafe for User_ {
    type SafeColumns = Columns;
    fn safe_columns_tuple() -> Self::SafeColumns {
      (
        id,
        name,
        preferred_username,
        avatar,
        admin,
        banned,
        published,
        updated,
        matrix_user_id,
        actor_id,
        bio,
        local,
        banner,
        deleted,
      )
    }
  }
}

mod safe_type_alias_1 {
  use crate::ToSafe;
  use lemmy_db_schema::{schema::user_alias_1::columns::*, source::user::UserAlias1};

  type Columns = (
    id,
    name,
    preferred_username,
    avatar,
    admin,
    banned,
    published,
    updated,
    matrix_user_id,
    actor_id,
    bio,
    local,
    banner,
    deleted,
  );

  impl ToSafe for UserAlias1 {
    type SafeColumns = Columns;
    fn safe_columns_tuple() -> Self::SafeColumns {
      (
        id,
        name,
        preferred_username,
        avatar,
        admin,
        banned,
        published,
        updated,
        matrix_user_id,
        actor_id,
        bio,
        local,
        banner,
        deleted,
      )
    }
  }
}

mod safe_type_alias_2 {
  use crate::ToSafe;
  use lemmy_db_schema::{schema::user_alias_2::columns::*, source::user::UserAlias2};

  type Columns = (
    id,
    name,
    preferred_username,
    avatar,
    admin,
    banned,
    published,
    updated,
    matrix_user_id,
    actor_id,
    bio,
    local,
    banner,
    deleted,
  );

  impl ToSafe for UserAlias2 {
    type SafeColumns = Columns;
    fn safe_columns_tuple() -> Self::SafeColumns {
      (
        id,
        name,
        preferred_username,
        avatar,
        admin,
        banned,
        published,
        updated,
        matrix_user_id,
        actor_id,
        bio,
        local,
        banner,
        deleted,
      )
    }
  }
}

mod safe_settings_type {
  use crate::ToSafeSettings;
  use lemmy_db_schema::{schema::user_::columns::*, source::user::User_};

  type Columns = (
    id,
    name,
    preferred_username,
    email,
    avatar,
    admin,
    banned,
    published,
    updated,
    show_nsfw,
    theme,
    default_sort_type,
    default_listing_type,
    lang,
    show_avatars,
    send_notifications_to_email,
    matrix_user_id,
    actor_id,
    bio,
    local,
    last_refreshed_at,
    banner,
    deleted,
  );

  impl ToSafeSettings for User_ {
    type SafeSettingsColumns = Columns;
    fn safe_settings_columns_tuple() -> Self::SafeSettingsColumns {
      (
        id,
        name,
        preferred_username,
        email,
        avatar,
        admin,
        banned,
        published,
        updated,
        show_nsfw,
        theme,
        default_sort_type,
        default_listing_type,
        lang,
        show_avatars,
        send_notifications_to_email,
        matrix_user_id,
        actor_id,
        bio,
        local,
        last_refreshed_at,
        banner,
        deleted,
      )
    }
  }
}

pub trait UserSafeSettings_ {
  fn read(conn: &PgConnection, user_id: i32) -> Result<UserSafeSettings, Error>;
}

impl UserSafeSettings_ for UserSafeSettings {
  fn read(conn: &PgConnection, user_id: i32) -> Result<Self, Error> {
    user_
      .select(User_::safe_settings_columns_tuple())
      .filter(deleted.eq(false))
      .find(user_id)
      .first::<Self>(conn)
  }
}

impl Crud<UserForm> for User_ {
  fn read(conn: &PgConnection, user_id: i32) -> Result<Self, Error> {
    user_
      .filter(deleted.eq(false))
      .find(user_id)
      .first::<Self>(conn)
  }
  fn delete(conn: &PgConnection, user_id: i32) -> Result<usize, Error> {
    diesel::delete(user_.find(user_id)).execute(conn)
  }
  fn create(conn: &PgConnection, form: &UserForm) -> Result<Self, Error> {
    insert_into(user_).values(form).get_result::<Self>(conn)
  }
  fn update(conn: &PgConnection, user_id: i32, form: &UserForm) -> Result<Self, Error> {
    diesel::update(user_.find(user_id))
      .set(form)
      .get_result::<Self>(conn)
  }
}

impl ApubObject<UserForm> for User_ {
  fn read_from_apub_id(conn: &PgConnection, object_id: &Url) -> Result<Self, Error> {
    use lemmy_db_schema::schema::user_::dsl::*;
    user_
      .filter(deleted.eq(false))
      .filter(actor_id.eq(object_id))
      .first::<Self>(conn)
  }

  fn upsert(conn: &PgConnection, user_form: &UserForm) -> Result<User_, Error> {
    insert_into(user_)
      .values(user_form)
      .on_conflict(actor_id)
      .do_update()
      .set(user_form)
      .get_result::<Self>(conn)
  }
}

pub trait User {
  fn register(conn: &PgConnection, form: &UserForm) -> Result<User_, Error>;
  fn update_password(conn: &PgConnection, user_id: i32, new_password: &str)
    -> Result<User_, Error>;
  fn read_from_name(conn: &PgConnection, from_user_name: &str) -> Result<User_, Error>;
  fn add_admin(conn: &PgConnection, user_id: i32, added: bool) -> Result<User_, Error>;
  fn ban_user(conn: &PgConnection, user_id: i32, ban: bool) -> Result<User_, Error>;
  fn find_by_email_or_username(
    conn: &PgConnection,
    username_or_email: &str,
  ) -> Result<User_, Error>;
  fn find_by_username(conn: &PgConnection, username: &str) -> Result<User_, Error>;
  fn find_by_email(conn: &PgConnection, from_email: &str) -> Result<User_, Error>;
  fn get_profile_url(&self, hostname: &str) -> String;
  fn mark_as_updated(conn: &PgConnection, user_id: i32) -> Result<User_, Error>;
  fn delete_account(conn: &PgConnection, user_id: i32) -> Result<User_, Error>;
}

impl User for User_ {
  fn register(conn: &PgConnection, form: &UserForm) -> Result<Self, Error> {
    let mut edited_user = form.clone();
    let password_hash =
      hash(&form.password_encrypted, DEFAULT_COST).expect("Couldn't hash password");
    edited_user.password_encrypted = password_hash;

    Self::create(&conn, &edited_user)
  }

  // TODO do more individual updates like these
  fn update_password(conn: &PgConnection, user_id: i32, new_password: &str) -> Result<Self, Error> {
    let password_hash = hash(new_password, DEFAULT_COST).expect("Couldn't hash password");

    diesel::update(user_.find(user_id))
      .set((
        password_encrypted.eq(password_hash),
        updated.eq(naive_now()),
      ))
      .get_result::<Self>(conn)
  }

  fn read_from_name(conn: &PgConnection, from_user_name: &str) -> Result<Self, Error> {
    user_
      .filter(local.eq(true))
      .filter(deleted.eq(false))
      .filter(name.eq(from_user_name))
      .first::<Self>(conn)
  }

  fn add_admin(conn: &PgConnection, user_id: i32, added: bool) -> Result<Self, Error> {
    diesel::update(user_.find(user_id))
      .set(admin.eq(added))
      .get_result::<Self>(conn)
  }

  fn ban_user(conn: &PgConnection, user_id: i32, ban: bool) -> Result<Self, Error> {
    diesel::update(user_.find(user_id))
      .set(banned.eq(ban))
      .get_result::<Self>(conn)
  }

  fn find_by_email_or_username(
    conn: &PgConnection,
    username_or_email: &str,
  ) -> Result<Self, Error> {
    if is_email_regex(username_or_email) {
      Self::find_by_email(conn, username_or_email)
    } else {
      Self::find_by_username(conn, username_or_email)
    }
  }

  fn find_by_username(conn: &PgConnection, username: &str) -> Result<User_, Error> {
    user_
      .filter(deleted.eq(false))
      .filter(local.eq(true))
      .filter(name.ilike(username))
      .first::<User_>(conn)
  }

  fn find_by_email(conn: &PgConnection, from_email: &str) -> Result<User_, Error> {
    user_
      .filter(deleted.eq(false))
      .filter(local.eq(true))
      .filter(email.eq(from_email))
      .first::<User_>(conn)
  }

  fn get_profile_url(&self, hostname: &str) -> String {
    format!(
      "{}://{}/u/{}",
      Settings::get().get_protocol_string(),
      hostname,
      self.name
    )
  }

  fn mark_as_updated(conn: &PgConnection, user_id: i32) -> Result<User_, Error> {
    diesel::update(user_.find(user_id))
      .set((last_refreshed_at.eq(naive_now()),))
      .get_result::<Self>(conn)
  }

  fn delete_account(conn: &PgConnection, user_id: i32) -> Result<User_, Error> {
    diesel::update(user_.find(user_id))
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
  use crate::{establish_unpooled_connection, source::user::*, ListingType, SortType};

  #[test]
  fn test_crud() {
    let conn = establish_unpooled_connection();

    let new_user = UserForm {
      name: "thommy".into(),
      preferred_username: None,
      password_encrypted: "nope".into(),
      email: None,
      matrix_user_id: None,
      avatar: None,
      banner: None,
      admin: false,
      banned: Some(false),
      published: None,
      updated: None,
      show_nsfw: false,
      theme: "browser".into(),
      default_sort_type: SortType::Hot as i16,
      default_listing_type: ListingType::Subscribed as i16,
      lang: "browser".into(),
      show_avatars: true,
      send_notifications_to_email: false,
      actor_id: None,
      bio: None,
      local: true,
      private_key: None,
      public_key: None,
      last_refreshed_at: None,
    };

    let inserted_user = User_::create(&conn, &new_user).unwrap();

    let expected_user = User_ {
      id: inserted_user.id,
      name: "thommy".into(),
      preferred_username: None,
      password_encrypted: "nope".into(),
      email: None,
      matrix_user_id: None,
      avatar: None,
      banner: None,
      admin: false,
      banned: false,
      published: inserted_user.published,
      updated: None,
      show_nsfw: false,
      theme: "browser".into(),
      default_sort_type: SortType::Hot as i16,
      default_listing_type: ListingType::Subscribed as i16,
      lang: "browser".into(),
      show_avatars: true,
      send_notifications_to_email: false,
      actor_id: inserted_user.actor_id.to_owned(),
      bio: None,
      local: true,
      private_key: None,
      public_key: None,
      last_refreshed_at: inserted_user.published,
      deleted: false,
    };

    let read_user = User_::read(&conn, inserted_user.id).unwrap();
    let updated_user = User_::update(&conn, inserted_user.id, &new_user).unwrap();
    let num_deleted = User_::delete(&conn, inserted_user.id).unwrap();

    assert_eq!(expected_user, read_user);
    assert_eq!(expected_user, inserted_user);
    assert_eq!(expected_user, updated_user);
    assert_eq!(1, num_deleted);
  }
}
