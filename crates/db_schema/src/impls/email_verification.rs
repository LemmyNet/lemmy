use crate::{
  newtypes::LocalUserId,
  schema::email_verification::dsl::{
    email_verification,
    local_user_id,
    published,
    verification_token,
  },
  source::email_verification::{EmailVerification, EmailVerificationForm},
  utils::{DbPool, GetConn},
};
use diesel::{
  dsl::{now, IntervalDsl},
  insert_into,
  result::Error,
  ExpressionMethods,
  QueryDsl,
};
use diesel_async::RunQueryDsl;

impl EmailVerification {
  pub async fn create(
    mut pool: &mut impl GetConn,
    form: &EmailVerificationForm,
  ) -> Result<Self, Error> {
    let conn = &mut *pool.get_conn().await?;
    insert_into(email_verification)
      .values(form)
      .get_result::<Self>(conn)
      .await
  }

  pub async fn read_for_token(mut pool: &mut impl GetConn, token: &str) -> Result<Self, Error> {
    let conn = &mut *pool.get_conn().await?;
    email_verification
      .filter(verification_token.eq(token))
      .filter(published.gt(now - 7.days()))
      .first::<Self>(conn)
      .await
  }
  pub async fn delete_old_tokens_for_local_user(
    mut pool: &mut impl GetConn,
    local_user_id_: LocalUserId,
  ) -> Result<usize, Error> {
    let conn = &mut *pool.get_conn().await?;
    diesel::delete(email_verification.filter(local_user_id.eq(local_user_id_)))
      .execute(conn)
      .await
  }
}
