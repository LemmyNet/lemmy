use super::*;
use crate::schema::password_reset_request;
use crate::schema::password_reset_request::dsl::*;
use sha2::{Digest, Sha256};

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
    hasher.input(token);
    let token_hash: String = PasswordResetRequest::bytes_to_hex(hasher.result().to_vec());

    let form = PasswordResetRequestForm {
      user_id: from_user_id,
      token_encrypted: token_hash,
    };

    Self::create(&conn, &form)
  }
  pub fn read_from_token(conn: &PgConnection, token: &str) -> Result<Self, Error> {
    let mut hasher = Sha256::new();
    hasher.input(token);
    let token_hash: String = PasswordResetRequest::bytes_to_hex(hasher.result().to_vec());
    password_reset_request
      .filter(token_encrypted.eq(token_hash))
      .filter(published.gt(now - 1.days()))
      .first::<Self>(conn)
  }

  fn bytes_to_hex(bytes: Vec<u8>) -> String {
    let mut str = String::new();
    for byte in bytes {
      str = format!("{}{:02x}", str, byte);
    }
    str
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
    };

    let inserted_user = User_::create(&conn, &new_user).unwrap();

    let token = "nope";
    let token_encrypted_ = "ca3704aa0b06f5954c79ee837faa152d84d6b2d42838f0637a15eda8337dbdce";

    let inserted_password_reset_request =
      PasswordResetRequest::create_token(&conn, inserted_user.id, token).unwrap();

    let expected_password_reset_request = PasswordResetRequest {
      id: inserted_password_reset_request.id,
      user_id: inserted_user.id,
      token_encrypted: token_encrypted_.to_string(),
      published: inserted_password_reset_request.published,
    };

    let read_password_reset_request = PasswordResetRequest::read_from_token(&conn, token).unwrap();
    let num_deleted = User_::delete(&conn, inserted_user.id).unwrap();

    assert_eq!(expected_password_reset_request, read_password_reset_request);
    assert_eq!(
      expected_password_reset_request,
      inserted_password_reset_request
    );
    assert_eq!(1, num_deleted);
  }
}
