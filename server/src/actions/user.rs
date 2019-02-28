extern crate diesel;
use schema::user_;
use diesel::*;
use diesel::result::Error;
use schema::user_::dsl::*;
use Crud;

#[derive(Queryable, PartialEq, Debug)]
pub struct User_ {
  pub id: i32,
  pub name: String,
  pub password_encrypted: String,
  pub email: Option<String>,
  pub icon: Option<Vec<u8>>,
  pub start_time: chrono::NaiveDateTime
}

#[derive(Insertable)]
#[table_name="user_"]
pub struct NewUser<'a> {
    pub name: &'a str,
    pub password_encrypted: &'a str,
    pub email: Option<&'a str>,
}

impl Crud for User_ {
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

// fn create<NewUser>(conn: &PgConnection, mut new_user: NewUser) -> Result<User_, Error> {
}

pub fn create(conn: &PgConnection, mut new_user: NewUser) -> Result<User_, Error> {
  new_user.password_encrypted = "here";
    // new_user.password_encrypted;
    insert_into(user_)
      .values(new_user)
      .get_result(conn)
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
      password_encrypted: "nope".into(),
      email: None
    };

    let inserted_user = create(&conn, new_user).unwrap();

    let expected_user = User_ {
      id: inserted_user.id,
      name: "thom".into(),
      password_encrypted: "here".into(),
      email: None,
      icon: None,
      start_time: inserted_user.start_time
    };

    let read_user = User_::read(&conn, inserted_user.id);
    let num_deleted = User_::delete(&conn, inserted_user.id);
    
    assert_eq!(expected_user, read_user);
    assert_eq!(expected_user, inserted_user);
    assert_eq!(1, num_deleted);

  }
}
