use crate::{
  newtypes::LocalUserId,
  source::oauth_account::{OAuthAccount, OAuthAccountInsertForm},
  utils::{get_conn, DbPool},
};
use diesel::{insert_into, result::Error, ExpressionMethods, QueryDsl};
use diesel_async::RunQueryDsl;
use lemmy_db_schema_file::schema::{oauth_account, oauth_account::dsl::local_user_id};

impl OAuthAccount {
  pub async fn create(pool: &mut DbPool<'_>, form: &OAuthAccountInsertForm) -> Result<Self, Error> {
    let conn = &mut get_conn(pool).await?;
    insert_into(oauth_account::table)
      .values(form)
      .get_result::<Self>(conn)
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
