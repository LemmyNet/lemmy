use crate::{
  schema::auth_refresh_token::dsl::{auth_refresh_token, token},
  source::auth_refresh_token::{
    AuthRefreshToken,
    AuthRefreshTokenCreateForm,
    AuthRefreshTokenUpdateForm,
  },
  utils::{get_conn, DbPool},
};
use diesel::{dsl::insert_into, result::Error, ExpressionMethods, QueryDsl};
use diesel_async::RunQueryDsl;

impl AuthRefreshToken {
  pub async fn create(
    pool: &mut DbPool<'_>,
    form: &AuthRefreshTokenCreateForm,
  ) -> Result<Self, Error> {
    let conn = &mut get_conn(pool).await?;
    insert_into(auth_refresh_token)
      .values(form)
      .get_result::<Self>(conn)
      .await
  }

  pub async fn update_token(
    pool: &mut DbPool<'_>,
    input_token: &str,
    form: &AuthRefreshTokenUpdateForm,
  ) -> Result<AuthRefreshToken, Error> {
    let conn = &mut get_conn(pool).await?;
    diesel::update(auth_refresh_token.filter(token.eq(input_token)))
      .set(form)
      .get_result::<Self>(conn)
      .await
  }
  pub async fn read_from_token(
    pool: &mut DbPool<'_>,
    input_token: &str,
  ) -> Result<AuthRefreshToken, Error> {
    let conn = &mut get_conn(pool).await?;
    auth_refresh_token
      .filter(token.eq(input_token))
      .first::<Self>(conn)
      .await
  }
}
