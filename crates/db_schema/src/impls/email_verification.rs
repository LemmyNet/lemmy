use crate::{newtypes::LocalUserId, source::email_verification::*};
use diesel::{
  dsl::*,
  insert_into,
  result::Error,
  ExpressionMethods,
  PgConnection,
  QueryDsl,
  RunQueryDsl,
};

impl EmailVerification {
  pub fn create(conn: &mut PgConnection, form: &EmailVerificationForm) -> Result<Self, Error> {
    use crate::schema::email_verification::dsl::*;
    insert_into(email_verification)
      .values(form)
      .get_result::<Self>(conn)
  }

  pub fn read_for_token(conn: &mut PgConnection, token: &str) -> Result<Self, Error> {
    use crate::schema::email_verification::dsl::*;
    email_verification
      .filter(verification_token.eq(token))
      .filter(published.gt(now - 7.days()))
      .first::<Self>(conn)
  }
  pub fn delete_old_tokens_for_local_user(
    conn: &mut PgConnection,
    local_user_id_: LocalUserId,
  ) -> Result<usize, Error> {
    use crate::schema::email_verification::dsl::*;
    diesel::delete(email_verification.filter(local_user_id.eq(local_user_id_))).execute(conn)
  }
}
