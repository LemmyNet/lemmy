use super::*;
use crate::schema::password_reset_request;
use crate::schema::password_reset_request::dsl::*;
use crypto::digest::Digest;
use crypto::sha2::Sha256;

#[derive(Queryable, Identifiable, PartialEq, Debug)]
#[table_name = "password_reset_request"]
pub struct PasswordResetRequest {
  pub id: i32,
  pub user_id: i32,
  pub token_encrypted: String,
  pub published: chrono::NaiveDateTime,
}

#[derive(Insertable, AsChangeset, Clone)]
#[table_name = "password_reset_request"]
pub struct PasswordResetRequestForm {
  pub user_id: i32,
  pub token_encrypted: String,
}

impl Crud<PasswordResetRequestForm> for PasswordResetRequest {
  fn read(conn: &PgConnection, password_reset_request_id: i32) -> Result<Self, Error> {
    use crate::schema::password_reset_request::dsl::*;
    password_reset_request
      .find(password_reset_request_id)
      .first::<Self>(conn)
  }
  fn delete(conn: &PgConnection, password_reset_request_id: i32) -> Result<usize, Error> {
    diesel::delete(password_reset_request.find(password_reset_request_id)).execute(conn)
  }
  fn create(conn: &PgConnection, form: &PasswordResetRequestForm) -> Result<Self, Error> {
    insert_into(password_reset_request)
      .values(form)
      .get_result::<Self>(conn)
  }
  fn update(
    conn: &PgConnection,
    password_reset_request_id: i32,
    form: &PasswordResetRequestForm,
  ) -> Result<Self, Error> {
    diesel::update(password_reset_request.find(password_reset_request_id))
      .set(form)
      .get_result::<Self>(conn)
  }
}

impl PasswordResetRequest {
  pub fn create_token(conn: &PgConnection, from_user_id: i32, token: &str) -> Result<Self, Error> {
    let mut hasher = Sha256::new();
    hasher.input_str(token);
    let token_hash = hasher.result_str();

    let form = PasswordResetRequestForm {
      user_id: from_user_id,
      token_encrypted: token_hash,
    };

    Self::create(&conn, &form)
  }
  pub fn read_from_token(conn: &PgConnection, token: &str) -> Result<Self, Error> {
    let mut hasher = Sha256::new();
    hasher.input_str(token);
    let token_hash = hasher.result_str();
    password_reset_request
      .filter(token_encrypted.eq(token_hash))
      .filter(published.gt(now - 1.days()))
      .first::<Self>(conn)
  }
}

#[cfg(test)]
mod tests {
  use super::super::user::*;
  use super::*;

  #[test]
  fn test_crud() {
    let conn = establish_connection();

    let new_user = UserForm {
      name: "thommy prw".into(),
      fedi_name: "rrf".into(),
      preferred_username: None,
      password_encrypted: "nope".into(),
      email: None,
      admin: false,
      banned: false,
      updated: None,
      show_nsfw: false,
      theme: "darkly".into(),
      default_sort_type: SortType::Hot as i16,
      default_listing_type: ListingType::Subscribed as i16,
      lang: "browser".into(),
    };

    let inserted_user = User_::create(&conn, &new_user).unwrap();

    let new_password_reset_request = PasswordResetRequestForm {
      user_id: inserted_user.id,
      token_encrypted: "no".into(),
    };

    let inserted_password_reset_request =
      PasswordResetRequest::create(&conn, &new_password_reset_request).unwrap();

    let expected_password_reset_request = PasswordResetRequest {
      id: inserted_password_reset_request.id,
      user_id: inserted_user.id,
      token_encrypted: "no".into(),
      published: inserted_password_reset_request.published,
    };

    let read_password_reset_request =
      PasswordResetRequest::read(&conn, inserted_password_reset_request.id).unwrap();
    let num_deleted = User_::delete(&conn, inserted_user.id).unwrap();

    assert_eq!(expected_password_reset_request, read_password_reset_request);
    assert_eq!(
      expected_password_reset_request,
      inserted_password_reset_request
    );
    assert_eq!(1, num_deleted);
  }
}
