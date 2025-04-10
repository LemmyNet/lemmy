use crate::{
  diesel::{ExpressionMethods, QueryDsl},
  newtypes::LocalUserId,
  source::login_token::{LoginToken, LoginTokenCreateForm},
  utils::{get_conn, DbPool},
};
use diesel::{delete, dsl::exists, insert_into, select};
use diesel_async::RunQueryDsl;
use lemmy_db_schema_file::schema::login_token::{dsl::login_token, user_id};
use lemmy_utils::error::{LemmyErrorExt, LemmyErrorType, LemmyResult};

impl LoginToken {
  pub async fn create(pool: &mut DbPool<'_>, form: LoginTokenCreateForm) -> LemmyResult<Self> {
    let conn = &mut get_conn(pool).await?;
    insert_into(login_token)
      .values(form)
      .get_result::<Self>(conn)
      .await
      .with_lemmy_type(LemmyErrorType::CouldntCreateLoginToken)
  }

  /// Check if the given token is valid for user.
  pub async fn validate(
    pool: &mut DbPool<'_>,
    user_id_: LocalUserId,
    token_: &str,
  ) -> LemmyResult<()> {
    let conn = &mut get_conn(pool).await?;
    select(exists(
      login_token.find(token_).filter(user_id.eq(user_id_)),
    ))
    .get_result::<bool>(conn)
    .await?
    .then_some(())
    .ok_or(LemmyErrorType::NotLoggedIn.into())
  }

  pub async fn list(pool: &mut DbPool<'_>, user_id_: LocalUserId) -> LemmyResult<Vec<LoginToken>> {
    let conn = &mut get_conn(pool).await?;

    login_token
      .filter(user_id.eq(user_id_))
      .get_results(conn)
      .await
      .with_lemmy_type(LemmyErrorType::NotFound)
  }

  /// Invalidate specific token on user logout.
  pub async fn invalidate(pool: &mut DbPool<'_>, token_: &str) -> LemmyResult<usize> {
    let conn = &mut get_conn(pool).await?;
    delete(login_token.find(token_))
      .execute(conn)
      .await
      .with_lemmy_type(LemmyErrorType::Deleted)
  }

  /// Invalidate all logins of given user on password reset/change, account deletion or site ban.
  pub async fn invalidate_all(pool: &mut DbPool<'_>, user_id_: LocalUserId) -> LemmyResult<usize> {
    let conn = &mut get_conn(pool).await?;
    delete(login_token.filter(user_id.eq(user_id_)))
      .execute(conn)
      .await
      .with_lemmy_type(LemmyErrorType::Deleted)
  }
}
