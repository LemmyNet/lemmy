use crate::{
  newtypes::{LocalUserId, OAuthProviderId},
  schema::{oauth_account, oauth_account::dsl::local_user_id},
  source::oauth_account::{OAuthAccount, OAuthAccountInsertForm},
  utils::{get_conn, DbPool},
};
use diesel::{
  dsl::{exists, insert_into},
  result::Error,
  select,
  ExpressionMethods,
  QueryDsl,
};
use diesel_async::RunQueryDsl;

impl OAuthAccount {
  pub async fn read(
    pool: &mut DbPool<'_>,
    for_oauth_provider_id: OAuthProviderId,
    for_local_user_id: LocalUserId,
  ) -> Result<bool, Error> {
    let conn = &mut get_conn(pool).await?;
    select(exists(
      oauth_account::table.find((for_oauth_provider_id, for_local_user_id)),
    ))
    .get_result(conn)
    .await
  }

  pub async fn create(pool: &mut DbPool<'_>, form: &OAuthAccountInsertForm) -> Result<Self, Error> {
    let conn = &mut get_conn(pool).await?;
    insert_into(oauth_account::table)
      .values(form)
      .get_result::<Self>(conn)
      .await
  }

  pub async fn delete(
    pool: &mut DbPool<'_>,
    for_oauth_provider_id: OAuthProviderId,
    for_local_user_id: LocalUserId,
  ) -> Result<usize, Error> {
    let conn = &mut get_conn(pool).await?;
    diesel::delete(oauth_account::table.find((for_oauth_provider_id, for_local_user_id)))
      .execute(conn)
      .await
  }

  pub async fn delete_user_accounts(
    pool: &mut DbPool<'_>,
    for_local_user_id: LocalUserId,
  ) -> Result<usize, Error> {
    let conn = &mut get_conn(pool).await?;

    diesel::delete(oauth_account::table.filter(local_user_id.eq(for_local_user_id)))
      .execute(conn)
      .await
  }
}
