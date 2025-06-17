use crate::{
  newtypes::LocalUserId,
  source::email_verification::{EmailVerification, EmailVerificationForm},
  utils::{get_conn, now, DbPool},
};
use diesel::{dsl::IntervalDsl, insert_into, ExpressionMethods, QueryDsl};
use diesel_async::RunQueryDsl;
use lemmy_db_schema_file::schema::email_verification;
use lemmy_utils::error::{LemmyErrorExt, LemmyErrorType, LemmyResult};

impl EmailVerification {
  pub async fn create(pool: &mut DbPool<'_>, form: &EmailVerificationForm) -> LemmyResult<Self> {
    let conn = &mut get_conn(pool).await?;
    insert_into(email_verification::table)
      .values(form)
      .get_result(conn)
      .await
      .with_lemmy_type(LemmyErrorType::CouldntCreateEmailVerification)
  }

  pub async fn read_for_token(pool: &mut DbPool<'_>, token: &str) -> LemmyResult<Self> {
    let conn = &mut get_conn(pool).await?;
    email_verification::table
      .filter(email_verification::verification_token.eq(token))
      .filter(email_verification::published_at.gt(now() - 7.days()))
      .first(conn)
      .await
      .with_lemmy_type(LemmyErrorType::NotFound)
  }
  pub async fn delete_old_tokens_for_local_user(
    pool: &mut DbPool<'_>,
    local_user_id_: LocalUserId,
  ) -> LemmyResult<usize> {
    let conn = &mut get_conn(pool).await?;
    diesel::delete(
      email_verification::table.filter(email_verification::local_user_id.eq(local_user_id_)),
    )
    .execute(conn)
    .await
    .with_lemmy_type(LemmyErrorType::Deleted)
  }
}
