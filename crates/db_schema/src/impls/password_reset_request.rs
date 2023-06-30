use crate::{
  newtypes::LocalUserId,
  schema::password_reset_request::dsl::{
    local_user_id,
    password_reset_request,
    published,
    token_encrypted,
  },
  source::password_reset_request::{PasswordResetRequest, PasswordResetRequestForm},
  traits::Crud,
  utils::DbConn,
};
use diesel::{
  dsl::{insert_into, now, IntervalDsl},
  result::Error,
  ExpressionMethods,
  QueryDsl,
};
use diesel_async::RunQueryDsl;
use sha2::{Digest, Sha256};

#[async_trait]
impl Crud for PasswordResetRequest {
  type InsertForm = PasswordResetRequestForm;
  type UpdateForm = PasswordResetRequestForm;
  type IdType = i32;
  async fn read(conn: &mut DbConn, password_reset_request_id: i32) -> Result<Self, Error> {
    password_reset_request
      .find(password_reset_request_id)
      .first::<Self>(conn)
      .await
  }
  async fn create(conn: &mut DbConn, form: &PasswordResetRequestForm) -> Result<Self, Error> {
    insert_into(password_reset_request)
      .values(form)
      .get_result::<Self>(conn)
      .await
  }
  async fn update(
    conn: &mut DbConn,
    password_reset_request_id: i32,
    form: &PasswordResetRequestForm,
  ) -> Result<Self, Error> {
    diesel::update(password_reset_request.find(password_reset_request_id))
      .set(form)
      .get_result::<Self>(conn)
      .await
  }
}

impl PasswordResetRequest {
  pub async fn create_token(
    conn: &mut DbConn,
    from_local_user_id: LocalUserId,
    token: &str,
  ) -> Result<PasswordResetRequest, Error> {
    let mut hasher = Sha256::new();
    hasher.update(token);
    let token_hash: String = bytes_to_hex(hasher.finalize().to_vec());

    let form = PasswordResetRequestForm {
      local_user_id: from_local_user_id,
      token_encrypted: token_hash,
    };

    Self::create(conn, &form).await
  }
  pub async fn read_from_token(
    conn: &mut DbConn,
    token: &str,
  ) -> Result<PasswordResetRequest, Error> {
    let mut hasher = Sha256::new();
    hasher.update(token);
    let token_hash: String = bytes_to_hex(hasher.finalize().to_vec());
    password_reset_request
      .filter(token_encrypted.eq(token_hash))
      .filter(published.gt(now - 1.days()))
      .first::<Self>(conn)
      .await
  }

  pub async fn get_recent_password_resets_count(
    conn: &mut DbConn,
    user_id: LocalUserId,
  ) -> Result<i64, Error> {
    password_reset_request
      .filter(local_user_id.eq(user_id))
      .filter(published.gt(now - 1.days()))
      .count()
      .get_result(conn)
      .await
  }
}

fn bytes_to_hex(bytes: Vec<u8>) -> String {
  let mut str = String::new();
  for byte in bytes {
    str = format!("{str}{byte:02x}");
  }
  str
}

#[cfg(test)]
mod tests {
  use crate::{
    source::{
      instance::Instance,
      local_user::{LocalUser, LocalUserInsertForm},
      password_reset_request::PasswordResetRequest,
      person::{Person, PersonInsertForm},
    },
    traits::Crud,
    utils::build_db_conn_for_tests,
  };
  use serial_test::serial;

  #[tokio::test]
  #[serial]
  async fn test_crud() {
    let conn = &mut build_db_conn_for_tests().await;

    let inserted_instance = Instance::read_or_create(conn, "my_domain.tld".to_string())
      .await
      .unwrap();

    let new_person = PersonInsertForm::builder()
      .name("thommy prw".into())
      .public_key("pubkey".to_string())
      .instance_id(inserted_instance.id)
      .build();

    let inserted_person = Person::create(conn, &new_person).await.unwrap();

    let new_local_user = LocalUserInsertForm::builder()
      .person_id(inserted_person.id)
      .password_encrypted("pass".to_string())
      .build();

    let inserted_local_user = LocalUser::create(conn, &new_local_user).await.unwrap();

    let token = "nope";
    let token_encrypted_ = "ca3704aa0b06f5954c79ee837faa152d84d6b2d42838f0637a15eda8337dbdce";

    let inserted_password_reset_request =
      PasswordResetRequest::create_token(conn, inserted_local_user.id, token)
        .await
        .unwrap();

    let expected_password_reset_request = PasswordResetRequest {
      id: inserted_password_reset_request.id,
      local_user_id: inserted_local_user.id,
      token_encrypted: token_encrypted_.to_string(),
      published: inserted_password_reset_request.published,
    };

    let read_password_reset_request = PasswordResetRequest::read_from_token(conn, token)
      .await
      .unwrap();
    let num_deleted = Person::delete(conn, inserted_person.id).await.unwrap();
    Instance::delete(conn, inserted_instance.id).await.unwrap();

    assert_eq!(expected_password_reset_request, read_password_reset_request);
    assert_eq!(
      expected_password_reset_request,
      inserted_password_reset_request
    );
    assert_eq!(1, num_deleted);
  }
}
