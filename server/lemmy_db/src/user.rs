use crate::{
  naive_now,
  schema::{user_, user_::dsl::*},
  Crud,
};
use bcrypt::{hash, DEFAULT_COST};
use diesel::{dsl::*, result::Error, *};

#[derive(Clone, Queryable, Identifiable, PartialEq, Debug)]
#[table_name = "user_"]
pub struct User_ {
  pub id: i32,
  pub name: String,
  pub preferred_username: Option<String>,
  pub password_encrypted: String,
  pub email: Option<String>,
  pub avatar: Option<String>,
  pub admin: bool,
  pub banned: bool,
  pub published: chrono::NaiveDateTime,
  pub updated: Option<chrono::NaiveDateTime>,
  pub show_nsfw: bool,
  pub theme: String,
  pub default_sort_type: i16,
  pub default_listing_type: i16,
  pub lang: String,
  pub show_avatars: bool,
  pub send_notifications_to_email: bool,
  pub matrix_user_id: Option<String>,
  pub actor_id: String,
  pub bio: Option<String>,
  pub local: bool,
  pub private_key: Option<String>,
  pub public_key: Option<String>,
  pub last_refreshed_at: chrono::NaiveDateTime,
}

#[derive(Insertable, AsChangeset, Clone, Debug)]
#[table_name = "user_"]
pub struct UserForm {
  pub name: String,
  pub preferred_username: Option<String>,
  pub password_encrypted: String,
  pub admin: bool,
  pub banned: bool,
  pub email: Option<String>,
  pub avatar: Option<String>,
  pub updated: Option<chrono::NaiveDateTime>,
  pub show_nsfw: bool,
  pub theme: String,
  pub default_sort_type: i16,
  pub default_listing_type: i16,
  pub lang: String,
  pub show_avatars: bool,
  pub send_notifications_to_email: bool,
  pub matrix_user_id: Option<String>,
  pub actor_id: String,
  pub bio: Option<String>,
  pub local: bool,
  pub private_key: Option<String>,
  pub public_key: Option<String>,
  pub last_refreshed_at: Option<chrono::NaiveDateTime>,
}

impl Crud<UserForm> for User_ {
  fn read(conn: &PgConnection, user_id: i32) -> Result<Self, Error> {
    user_.find(user_id).first::<Self>(conn)
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

impl User_ {
  pub fn register(conn: &PgConnection, form: &UserForm) -> Result<Self, Error> {
    let mut edited_user = form.clone();
    let password_hash =
      hash(&form.password_encrypted, DEFAULT_COST).expect("Couldn't hash password");
    edited_user.password_encrypted = password_hash;

    Self::create(&conn, &edited_user)
  }

  // TODO do more individual updates like these
  pub fn update_password(
    conn: &PgConnection,
    user_id: i32,
    new_password: &str,
  ) -> Result<Self, Error> {
    let password_hash = hash(new_password, DEFAULT_COST).expect("Couldn't hash password");

    diesel::update(user_.find(user_id))
      .set((
        password_encrypted.eq(password_hash),
        updated.eq(naive_now()),
      ))
      .get_result::<Self>(conn)
  }

  pub fn read_from_name(conn: &PgConnection, from_user_name: &str) -> Result<Self, Error> {
    user_.filter(name.eq(from_user_name)).first::<Self>(conn)
  }

  pub fn add_admin(conn: &PgConnection, user_id: i32, added: bool) -> Result<Self, Error> {
    diesel::update(user_.find(user_id))
      .set(admin.eq(added))
      .get_result::<Self>(conn)
  }

  pub fn ban_user(conn: &PgConnection, user_id: i32, ban: bool) -> Result<Self, Error> {
    diesel::update(user_.find(user_id))
      .set(banned.eq(ban))
      .get_result::<Self>(conn)
  }

  pub fn read_from_actor_id(conn: &PgConnection, object_id: &str) -> Result<Self, Error> {
    use crate::schema::user_::dsl::*;
    user_.filter(actor_id.eq(object_id)).first::<Self>(conn)
  }
}

impl User_ {
  pub fn find_by_username(conn: &PgConnection, username: &str) -> Result<User_, Error> {
    user_.filter(name.eq(username)).first::<User_>(conn)
  }

  pub fn find_by_email(conn: &PgConnection, from_email: &str) -> Result<User_, Error> {
    user_.filter(email.eq(from_email)).first::<User_>(conn)
  }

  pub fn get_profile_url(&self, hostname: &str) -> String {
    format!("https://{}/u/{}", hostname, self.name)
  }
}

#[cfg(test)]
mod tests {
  use crate::{tests::establish_unpooled_connection, user::*, ListingType, SortType};

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
      admin: false,
      banned: false,
      updated: None,
      show_nsfw: false,
      theme: "darkly".into(),
      default_sort_type: SortType::Hot as i16,
      default_listing_type: ListingType::Subscribed as i16,
      lang: "browser".into(),
      show_avatars: true,
      send_notifications_to_email: false,
      actor_id: "http://fake.com".into(),
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
      admin: false,
      banned: false,
      published: inserted_user.published,
      updated: None,
      show_nsfw: false,
      theme: "darkly".into(),
      default_sort_type: SortType::Hot as i16,
      default_listing_type: ListingType::Subscribed as i16,
      lang: "browser".into(),
      show_avatars: true,
      send_notifications_to_email: false,
      actor_id: "http://fake.com".into(),
      bio: None,
      local: true,
      private_key: None,
      public_key: None,
      last_refreshed_at: inserted_user.published,
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
