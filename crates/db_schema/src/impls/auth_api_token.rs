use crate::{
  schema::auth_api_token::dsl::{auth_api_token, token},
  source::auth_api_token::{AuthApiToken, AuthApiTokenCreateForm, AuthApiTokenUpdateForm},
  utils::{get_conn, DbPool},
};
use diesel::{dsl::insert_into, result::Error, ExpressionMethods, QueryDsl};
use diesel_async::RunQueryDsl;

impl AuthApiToken {
  pub async fn create(pool: &mut DbPool<'_>, form: &AuthApiTokenCreateForm) -> Result<Self, Error> {
    let conn = &mut get_conn(pool).await?;
    insert_into(auth_api_token)
      .values(form)
      .get_result::<Self>(conn)
      .await
  }

  pub async fn update_token(
    pool: &mut DbPool<'_>,
    input_token: &str,
    form: &AuthApiTokenUpdateForm,
  ) -> Result<AuthApiToken, Error> {
    let conn = &mut get_conn(pool).await?;
    diesel::update(auth_api_token.filter(token.eq(input_token)))
      .set(form)
      .get_result::<Self>(conn)
      .await
  }
  pub async fn read_from_token(
    pool: &mut DbPool<'_>,
    input_token: &str,
  ) -> Result<AuthApiToken, Error> {
    let conn = &mut get_conn(pool).await?;
    auth_api_token
      .filter(token.eq(input_token))
      .first::<Self>(conn)
      .await
  }
}
