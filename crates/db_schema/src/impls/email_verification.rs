use crate::{
  newtypes::LocalUserId,
  schema::email_verification::dsl::{
    email_verification,
    local_user_id,
    published,
    verification_token,
  },
  source::email_verification::{EmailVerification, EmailVerificationForm},
  utils::{get_conn, DbPool},
};
use diesel::{
  dsl::{now, IntervalDsl},
  insert_into,
  result::Error,
  ExpressionMethods,
  QueryDsl, IntoSql, sql_types::Timestamptz,
};
use diesel_async::RunQueryDsl;

impl EmailVerification {
  pub async fn create(pool: &DbPool, form: &EmailVerificationForm) -> Result<Self, Error> {
    let conn = &mut get_conn(pool).await?;
    insert_into(email_verification)
      .values(form)
      .get_result::<Self>(conn)
      .await
  }

  pub async fn read_for_token(pool: &DbPool, token: &str) -> Result<Self, Error> {
    let conn = &mut get_conn(pool).await?;
    email_verification
      .filter(verification_token.eq(token))
      .filter(published.gt(now.into_sql::<Timestamptz>() - 7.days()))
      .first::<Self>(conn)
      .await
  }
  pub async fn delete_old_tokens_for_local_user(
    pool: &DbPool,
    local_user_id_: LocalUserId,
  ) -> Result<usize, Error> {
    let conn = &mut get_conn(pool).await?;
    diesel::delete(email_verification.filter(local_user_id.eq(local_user_id_)))
      .execute(conn)
      .await
  }
}
