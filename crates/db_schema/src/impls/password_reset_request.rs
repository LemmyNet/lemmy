use crate::{
  newtypes::LocalUserId,
  schema::password_reset_request::dsl::{local_user_id, password_reset_request, published, token},
  source::password_reset_request::{PasswordResetRequest, PasswordResetRequestForm},
  traits::Crud,
  utils::{get_conn, DbPool},
};
use diesel::{
  dsl::{insert_into, now, IntervalDsl},
  result::Error,
  ExpressionMethods,
  QueryDsl,
};
use diesel_async::RunQueryDsl;

#[async_trait]
impl Crud for PasswordResetRequest {
  type InsertForm = PasswordResetRequestForm;
  type UpdateForm = PasswordResetRequestForm;
  type IdType = i32;

  async fn create(pool: &mut DbPool<'_>, form: &PasswordResetRequestForm) -> Result<Self, Error> {
    let conn = &mut get_conn(pool).await?;
    insert_into(password_reset_request)
      .values(form)
      .get_result::<Self>(conn)
      .await
  }
  async fn update(
    pool: &mut DbPool<'_>,
    password_reset_request_id: i32,
    form: &PasswordResetRequestForm,
  ) -> Result<Self, Error> {
    let conn = &mut get_conn(pool).await?;
    diesel::update(password_reset_request.find(password_reset_request_id))
      .set(form)
      .get_result::<Self>(conn)
      .await
  }
}

impl PasswordResetRequest {
  pub async fn create_token(
    pool: &mut DbPool<'_>,
    from_local_user_id: LocalUserId,
    token_: String,
  ) -> Result<PasswordResetRequest, Error> {
    let form = PasswordResetRequestForm {
      local_user_id: from_local_user_id,
      token: token_,
    };

    Self::create(pool, &form).await
  }
  pub async fn read_from_token(
    pool: &mut DbPool<'_>,
    token_: &str,
  ) -> Result<PasswordResetRequest, Error> {
    let conn = &mut get_conn(pool).await?;
    password_reset_request
      .filter(token.eq(token_))
      .filter(published.gt(now - 1.days()))
      .first::<Self>(conn)
      .await
  }

  pub async fn get_recent_password_resets_count(
    pool: &mut DbPool<'_>,
    user_id: LocalUserId,
  ) -> Result<i64, Error> {
    let conn = &mut get_conn(pool).await?;
    password_reset_request
      .filter(local_user_id.eq(user_id))
      .filter(published.gt(now - 1.days()))
      .count()
      .get_result(conn)
      .await
  }
}

#[cfg(test)]
mod tests {
  #![allow(clippy::unwrap_used)]
  #![allow(clippy::indexing_slicing)]

  use crate::{
    source::{
      instance::Instance,
      local_user::{LocalUser, LocalUserInsertForm},
      password_reset_request::PasswordResetRequest,
      person::{Person, PersonInsertForm},
    },
    traits::Crud,
    utils::build_db_pool_for_tests,
  };
  use serial_test::serial;

  #[tokio::test]
  #[serial]
  async fn test_crud() {
    let pool = &build_db_pool_for_tests().await;
    let pool = &mut pool.into();

    let inserted_instance = Instance::read_or_create(pool, "my_domain.tld".to_string())
      .await
      .unwrap();

    let new_person = PersonInsertForm::builder()
      .name("thommy prw".into())
      .public_key("pubkey".to_string())
      .instance_id(inserted_instance.id)
      .build();

    let inserted_person = Person::create(pool, &new_person).await.unwrap();

    let new_local_user = LocalUserInsertForm::builder()
      .person_id(inserted_person.id)
      .password_encrypted("pass".to_string())
      .build();

    let inserted_local_user = LocalUser::create(pool, &new_local_user).await.unwrap();

    let token = "nope";

    let inserted_password_reset_request =
      PasswordResetRequest::create_token(pool, inserted_local_user.id, token.to_string())
        .await
        .unwrap();

    let expected_password_reset_request = PasswordResetRequest {
      id: inserted_password_reset_request.id,
      local_user_id: inserted_local_user.id,
      token: token.to_string(),
      published: inserted_password_reset_request.published,
    };

    let read_password_reset_request = PasswordResetRequest::read_from_token(pool, token)
      .await
      .unwrap();
    let num_deleted = Person::delete(pool, inserted_person.id).await.unwrap();
    Instance::delete(pool, inserted_instance.id).await.unwrap();

    assert_eq!(expected_password_reset_request, read_password_reset_request);
    assert_eq!(
      expected_password_reset_request,
      inserted_password_reset_request
    );
    assert_eq!(1, num_deleted);
  }
}
