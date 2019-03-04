extern crate diesel;
use schema::user_;
use diesel::*;
use diesel::result::Error;
use schema::user_::dsl::*;

#[derive(Queryable, PartialEq, Debug)]
pub struct User_ {
  pub id: i32,
  pub name: String,
  pub preferred_username: Option<String>,
  pub password_encrypted: String,
  pub email: Option<String>,
  pub icon: Option<Vec<u8>>,
  pub start_time: chrono::NaiveDateTime
}

#[derive(Insertable, AsChangeset, Clone, Copy)]
#[table_name="user_"]
pub struct NewUser<'a> {
    pub name: &'a str,
    pub preferred_username: Option<&'a str>,
    pub password_encrypted: &'a str,
    pub email: Option<&'a str>,
}

pub fn read(conn: &PgConnection, user_id: i32) -> User_ {
  user_.find(user_id)
    .first::<User_>(conn)
    .expect("Error in query")
}

pub fn delete(conn: &PgConnection, user_id: i32) -> usize {
  diesel::delete(user_.find(user_id))
    .execute(conn)
    .expect("Error deleting.")
}

pub fn create(conn: &PgConnection, new_user: &NewUser) -> Result<User_, Error> {
  let mut edited_user = new_user.clone();
  // Add the rust crypt
  edited_user.password_encrypted = "here";
    // edited_user.password_encrypted;
    insert_into(user_)
      .values(edited_user)
      .get_result::<User_>(conn)
}

pub fn update(conn: &PgConnection, user_id: i32, new_user: &NewUser) -> User_ {
  let mut edited_user = new_user.clone();
  edited_user.password_encrypted = "here";
  diesel::update(user_.find(user_id))
    .set(edited_user)
    .get_result::<User_>(conn)
    .expect(&format!("Unable to find user {}", user_id))
}

#[cfg(test)]
mod tests {
  use establish_connection;
  use super::*;
 #[test]
  fn test_crud() {
    let conn = establish_connection();
    
    let new_user = NewUser {
      name: "thom".into(),
      preferred_username: None,
      password_encrypted: "nope".into(),
      email: None
    };

    let inserted_user = create(&conn, &new_user).unwrap();

    let expected_user = User_ {
      id: inserted_user.id,
      name: "thom".into(),
      preferred_username: None,
      password_encrypted: "here".into(),
      email: None,
      icon: None,
      start_time: inserted_user.start_time
    };
    
    let read_user = read(&conn, inserted_user.id);
    let updated_user = update(&conn, inserted_user.id, &new_user);
    let num_deleted = delete(&conn, inserted_user.id);

    assert_eq!(expected_user, read_user);
    assert_eq!(expected_user, inserted_user);
    assert_eq!(expected_user, updated_user);
    assert_eq!(1, num_deleted);

  }
}
