use crate::{
  diesel::{ExpressionMethods, QueryDsl},
  newtypes::LocalUserId,
  schema::login_token::{dsl::login_token, token, user_id},
  source::login_token::{LoginToken, LoginTokenCreateForm},
  utils::{get_conn, DbPool},
};
use diesel::{delete, dsl::exists, insert_into, result::Error, select};
use diesel_async::RunQueryDsl;

impl LoginToken {
  pub async fn create(pool: &mut DbPool<'_>, form: LoginTokenCreateForm) -> Result<Self, Error> {
    let conn = &mut get_conn(pool).await?;
    insert_into(login_token)
      .values(form)
      .get_result::<Self>(conn)
      .await
  }

  /// Check if the given token is valid for user.
  pub async fn validate(
    pool: &mut DbPool<'_>,
    user_id_: LocalUserId,
    token_: &str,
  ) -> Result<bool, Error> {
    let conn = &mut get_conn(pool).await?;
    select(exists(
      login_token
        .filter(user_id.eq(user_id_))
        .filter(token.eq(token_)),
    ))
    .get_result(conn)
    .await
  }

  /// Invalidate specific token on user logout.
  pub async fn invalidate(pool: &mut DbPool<'_>, token_: &str) -> Result<usize, Error> {
    let conn = &mut get_conn(pool).await?;
    delete(login_token.filter(token.eq(token_)))
      .execute(conn)
      .await
  }

  /// Invalidate all logins of given user on password reset/change, account deletion or site ban.
  pub async fn invalidate_all(
    pool: &mut DbPool<'_>,
    user_id_: LocalUserId,
  ) -> Result<usize, Error> {
    let conn = &mut get_conn(pool).await?;
    delete(login_token.filter(user_id.eq(user_id_)))
      .execute(conn)
      .await
  }
}
