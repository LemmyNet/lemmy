use crate::{
  newtypes::LocalUserId,
  schema::email_verification::dsl::{
    email_verification,
    local_user_id,
    published,
    verification_token,
  },
  source::email_verification::{EmailVerification, EmailVerificationForm},
  utils::{DbPool, DbPoolRef, RunQueryDsl},
};
use diesel::{
  dsl::{now, IntervalDsl},
  insert_into,
  result::Error,
  ExpressionMethods,
  QueryDsl,
};

impl EmailVerification {
  pub async fn create(pool: DbPoolRef<'_>, form: &EmailVerificationForm) -> Result<Self, Error> {
    let conn = pool;
    insert_into(email_verification)
      .values(form)
      .get_result::<Self>(conn)
      .await
  }

  pub async fn read_for_token(pool: DbPoolRef<'_>, token: &str) -> Result<Self, Error> {
    let conn = pool;
    email_verification
      .filter(verification_token.eq(token))
      .filter(published.gt(now - 7.days()))
      .first::<Self>(conn)
      .await
  }
  pub async fn delete_old_tokens_for_local_user(
    pool: DbPoolRef<'_>,
    local_user_id_: LocalUserId,
  ) -> Result<usize, Error> {
    let conn = pool;
    diesel::delete(email_verification.filter(local_user_id.eq(local_user_id_)))
      .execute(conn)
      .await
  }
}
