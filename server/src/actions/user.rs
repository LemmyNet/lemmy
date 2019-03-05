extern crate diesel;
use schema::user_;
use diesel::*;
use diesel::result::Error;
use schema::user_::dsl::*;
use Crud;

#[derive(Queryable, Identifiable, PartialEq, Debug)]
#[table_name="user_"]
pub struct User_ {
  pub id: i32,
  pub name: String,
  pub preferred_username: Option<String>,
  pub password_encrypted: String,
  pub email: Option<String>,
  pub icon: Option<Vec<u8>>,
  pub published: chrono::NaiveDateTime,
  pub updated: Option<chrono::NaiveDateTime>
}

#[derive(Insertable, AsChangeset, Clone, Copy)]
#[table_name="user_"]
pub struct UserForm<'a> {
    pub name: &'a str,
    pub preferred_username: Option<&'a str>,
    pub password_encrypted: &'a str,
    pub email: Option<&'a str>,
    pub updated: Option<&'a chrono::NaiveDateTime>
}

impl<'a> Crud<UserForm<'a>> for User_ {
  fn read(conn: &PgConnection, user_id: i32) -> User_ {
    user_.find(user_id)
      .first::<User_>(conn)
      .expect("Error in query")
  }
  fn delete(conn: &PgConnection, user_id: i32) -> usize {
    diesel::delete(user_.find(user_id))
      .execute(conn)
      .expect("Error deleting.")
  }
  fn create(conn: &PgConnection, form: UserForm) -> Result<User_, Error> {
    let mut edited_user = form.clone();
    // Add the rust crypt
    edited_user.password_encrypted = "here";
      // edited_user.password_encrypted;
      insert_into(user_)
        .values(edited_user)
        .get_result::<User_>(conn)
  }
  fn update(conn: &PgConnection, user_id: i32, form: UserForm) -> User_ {
    let mut edited_user = form.clone();
    edited_user.password_encrypted = "here";
    diesel::update(user_.find(user_id))
      .set(edited_user)
      .get_result::<User_>(conn)
      .expect(&format!("Unable to find user {}", user_id))
  }
}

#[cfg(test)]
mod tests {
  use establish_connection;
  use super::{User_, UserForm};
  use Crud;
 #[test]
  fn test_crud() {
    let conn = establish_connection();
    
    let new_user = UserForm {
      name: "thom".into(),
      preferred_username: None,
      password_encrypted: "nope".into(),
      email: None,
      updated: None
    };

    let inserted_user = User_::create(&conn, new_user).unwrap();

    let expected_user = User_ {
      id: inserted_user.id,
      name: "thom".into(),
      preferred_username: None,
      password_encrypted: "here".into(),
      email: None,
      icon: None,
      published: inserted_user.published,
      updated: None
    };
    
    let read_user = User_::read(&conn, inserted_user.id);
    let updated_user = User_::update(&conn, inserted_user.id, new_user);
    let num_deleted = User_::delete(&conn, inserted_user.id);

    assert_eq!(expected_user, read_user);
    assert_eq!(expected_user, inserted_user);
    assert_eq!(expected_user, updated_user);
    assert_eq!(1, num_deleted);
  }
}
