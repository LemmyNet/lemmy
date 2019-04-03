use schema::user_;
use diesel::*;
use diesel::result::Error;
use schema::user_::dsl::*;
use serde::{Serialize, Deserialize};
use {Crud,is_email_regex};
use jsonwebtoken::{encode, decode, Header, Validation, TokenData};
use bcrypt::{DEFAULT_COST, hash};

#[derive(Queryable, Identifiable, PartialEq, Debug)]
#[table_name="user_"]
pub struct User_ {
  pub id: i32,
  pub name: String,
  pub fedi_name: String,
  pub preferred_username: Option<String>,
  pub password_encrypted: String,
  pub email: Option<String>,
  pub icon: Option<Vec<u8>>,
  pub published: chrono::NaiveDateTime,
  pub updated: Option<chrono::NaiveDateTime>
}

#[derive(Insertable, AsChangeset, Clone)]
#[table_name="user_"]
pub struct UserForm {
    pub name: String,
    pub fedi_name: String,
    pub preferred_username: Option<String>,
    pub password_encrypted: String,
    pub email: Option<String>,
    pub updated: Option<chrono::NaiveDateTime>
}

impl Crud<UserForm> for User_ {
  fn read(conn: &PgConnection, user_id: i32) -> Result<Self, Error> {
    user_.find(user_id)
      .first::<Self>(conn)
  }
  fn delete(conn: &PgConnection, user_id: i32) -> Result<usize, Error> {
    diesel::delete(user_.find(user_id))
      .execute(conn)
  }
  fn create(conn: &PgConnection, form: &UserForm) -> Result<Self, Error> {
    let mut edited_user = form.clone();
    let password_hash = hash(&form.password_encrypted, DEFAULT_COST)
      .expect("Couldn't hash password");
    edited_user.password_encrypted = password_hash;
    insert_into(user_)
      .values(edited_user)
      .get_result::<Self>(conn)
  }
  fn update(conn: &PgConnection, user_id: i32, form: &UserForm) -> Result<Self, Error> {
    let mut edited_user = form.clone();
    let password_hash = hash(&form.password_encrypted, DEFAULT_COST)
      .expect("Couldn't hash password");
    edited_user.password_encrypted = password_hash;
    diesel::update(user_.find(user_id))
      .set(edited_user)
      .get_result::<Self>(conn)
  }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
  pub id: i32,
  pub username: String,
  pub iss: String,
}

impl Claims {
  pub fn decode(jwt: &str) -> Result<TokenData<Claims>, jsonwebtoken::errors::Error> {
    let v = Validation {
      validate_exp: false,
      ..Validation::default()
    };
    decode::<Claims>(&jwt, "secret".as_ref(), &v)
  }
}

type Jwt = String;
impl User_ {
  pub fn jwt(&self) -> Jwt {
    let my_claims = Claims {
      id: self.id,
      username: self.name.to_owned(),
      iss: "rrf".to_string() // TODO this should come from config file
    };
    encode(&Header::default(), &my_claims, "secret".as_ref()).unwrap()
  }

  pub fn find_by_email_or_username(conn: &PgConnection, username_or_email: &str) -> Result<Self, Error> {
    if is_email_regex(username_or_email) {
      user_.filter(email.eq(username_or_email))
        .first::<User_>(conn)
    } else {
      user_.filter(name.eq(username_or_email))
        .first::<User_>(conn)
    }
  }

  pub fn find_by_jwt(conn: &PgConnection, jwt: &str) -> Result<Self, Error> {
    let claims: Claims = Claims::decode(&jwt).expect("Invalid token").claims;
    Self::read(&conn, claims.id)
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
      fedi_name: "rrf".into(),
      preferred_username: None,
      password_encrypted: "nope".into(),
      email: None,
      updated: None
    };

    let inserted_user = User_::create(&conn, &new_user).unwrap();

    let expected_user = User_ {
      id: inserted_user.id,
      name: "thom".into(),
      fedi_name: "rrf".into(),
      preferred_username: None,
      password_encrypted: "$2y$12$YXpNpYsdfjmed.QlYLvw4OfTCgyKUnKHc/V8Dgcf9YcVKHPaYXYYy".into(),
      email: None,
      icon: None,
      published: inserted_user.published,
      updated: None
    };
    
    let read_user = User_::read(&conn, inserted_user.id).unwrap();
    let updated_user = User_::update(&conn, inserted_user.id, &new_user).unwrap();
    let num_deleted = User_::delete(&conn, inserted_user.id).unwrap();

    assert_eq!(expected_user.id, read_user.id);
    assert_eq!(expected_user.id, inserted_user.id);
    assert_eq!(expected_user.id, updated_user.id);
    assert_eq!(1, num_deleted);
  }
}
