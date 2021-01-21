use crate::Crud;
use diesel::{dsl::*, result::Error, PgConnection, *};
use lemmy_db_schema::{schema::password_reset_request::dsl::*, source::password_reset_request::*};
use sha2::{Digest, Sha256};

impl Crud<PasswordResetRequestForm> for PasswordResetRequest {
  fn read(conn: &PgConnection, password_reset_request_id: i32) -> Result<Self, Error> {
    password_reset_request
      .find(password_reset_request_id)
      .first::<Self>(conn)
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

pub trait PasswordResetRequest_ {
  fn create_token(
    conn: &PgConnection,
    from_user_id: i32,
    token: &str,
  ) -> Result<PasswordResetRequest, Error>;
  fn read_from_token(conn: &PgConnection, token: &str) -> Result<PasswordResetRequest, Error>;
}

impl PasswordResetRequest_ for PasswordResetRequest {
  fn create_token(
    conn: &PgConnection,
    from_user_id: i32,
    token: &str,
  ) -> Result<PasswordResetRequest, Error> {
    let mut hasher = Sha256::new();
    hasher.update(token);
    let token_hash: String = bytes_to_hex(hasher.finalize().to_vec());

    let form = PasswordResetRequestForm {
      user_id: from_user_id,
      token_encrypted: token_hash,
    };

    Self::create(&conn, &form)
  }
  fn read_from_token(conn: &PgConnection, token: &str) -> Result<PasswordResetRequest, Error> {
    let mut hasher = Sha256::new();
    hasher.update(token);
    let token_hash: String = bytes_to_hex(hasher.finalize().to_vec());
    password_reset_request
      .filter(token_encrypted.eq(token_hash))
      .filter(published.gt(now - 1.days()))
      .first::<Self>(conn)
  }
}

fn bytes_to_hex(bytes: Vec<u8>) -> String {
  let mut str = String::new();
  for byte in bytes {
    str = format!("{}{:02x}", str, byte);
  }
  str
}

#[cfg(test)]
mod tests {
  use crate::{
    establish_unpooled_connection,
    source::password_reset_request::PasswordResetRequest_,
    Crud,
    ListingType,
    SortType,
  };
  use lemmy_db_schema::source::{password_reset_request::PasswordResetRequest, user::*};

  #[test]
  fn test_crud() {
    let conn = establish_unpooled_connection();

    let new_user = UserForm {
      name: "thommy prw".into(),
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
